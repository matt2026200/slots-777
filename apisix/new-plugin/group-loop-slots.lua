local redis_mod = require("resty.redis")
local core      = require("apisix.core")

local _M = { version = 0.4 }

-- ==========================================
-- 环形缓冲区核心配置
-- ==========================================
local RING_SIZE = 3          -- 环形槽位总数（3个槽位足以实现完美读写交替）
local current_slot = 1       -- 当前正在写入的槽位指针

-- 初始化环形数据结构
local ring_buffers = {}
for i = 1, RING_SIZE do
    ring_buffers[i] = {}     -- 每个槽位初始化一个独立的数组
end

local MAX_BATCH_SIZE = 500
local FLUSH_INTERVAL = 0.002 -- 2毫秒
local is_timer_running = false

-- ==========================================
-- 后台定时器：轮转指针并安全消费旧槽位
-- ==========================================
local function flush_ring_timer(premature)
    if premature then return end

    ----------------------------------------------------------------
    -- 核心优化：原子级指针滚动（彻底隔离读写）
    ----------------------------------------------------------------
    local consume_slot = current_slot -- 记录当前待消费的槽位
    
    -- 指针向前滚动一格 (1 -> 2 -> 3 -> 1)
    current_slot = (current_slot % RING_SIZE) + 1
    
    -- 重置定时器状态标志，允许后续新流入的请求在必要时拉起下一个定时器
    is_timer_running = false 

    -- 现在，所有的后续写入流量已经全流向了新的环形槽位！
    -- 我们可以安全地消费旧槽位 `consume_slot`，不需要担心任何并发夹带
    local batch = ring_buffers[consume_slot]
    local queue_len = #batch
    
    if queue_len == 0 then return end

    -- 执行 Redis Pipeline 交互
    local red = redis_mod:new()
    red:set_timeout(1000)
    local ok, err = red:connect("192.168.1.50", 6379)
    if not ok then
        core.log.error("Redis connect failed in ring timer: ", err)
        for i = 1, queue_len do batch[i].semaphore:post(false) end
        ring_buffers[consume_slot] = {} -- 释放内存空间
        return
    end

    red:init_pipeline(queue_len)
    for i = 1, queue_len do
        red:set("replay:uuid:" .. batch[i].uuid, "1", "EX", 30, "NX")
    end

    local results, err = red:commit_pipeline()
    red:set_keepalive(10000, 100)

    -- 结果分发并唤醒对应挂起的业务协程
    if not results then
        for i = 1, queue_len do batch[i].semaphore:post(false) end
    else
        for i = 1, queue_len do
            local res = results[i]
            if res and res ~= ngx.null and res == "OK" then
                batch[i].semaphore:post(true)
            else
                batch[i].semaphore:post(false)
            end
        end
    end

    ----------------------------------------------------------------
    -- 回收清理：清空已被消费完毕的槽位，留给下一轮轮转使用
    ----------------------------------------------------------------
    ring_buffers[consume_slot] = {}
end

-- ==========================================
-- 前端业务请求准入入口
-- ==========================================
function _M.rewrite(conf, ctx)
    local uuid = core.request.header(ctx, "X-HMAC-UUID")
    if not uuid then return 401 end

    local semaphore = require("ngx.semaphore").new()

    ----------------------------------------------------------------
    -- 写入线：永远只往当前激活的 current_slot 里追加
    ----------------------------------------------------------------
    local active_slot = current_slot
    table.insert(ring_buffers[active_slot], {
        uuid = uuid,
        semaphore = semaphore
    })

    -- 触发定时器检查
    if #ring_buffers[active_slot] >= MAX_BATCH_SIZE and not is_timer_running then
        is_timer_running = true
        ngx.timer.at(0, flush_ring_timer)
    elseif not is_timer_running then
        is_timer_running = true
        ngx.timer.at(FLUSH_INTERVAL, flush_ring_timer)
    end

    -- 协程挂起，静默等待当前请求在未来的定时器中被批量消费
    local wait_ok, is_valid_request = semaphore:wait(0.5)

    if not wait_ok or not is_valid_request then
        return 403, { message = "Replay detected or timeout." }
    end
    
    -- 验证通过，放行
end

return _M
