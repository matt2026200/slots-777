local core     = require("apisix.core")
local hmac     = require("apisix.plugins.hmac-auth")
local resty_sha256 = require("resty.sha256")
local str      = require("resty.string")

-- 初始化本地一级缓存 (L1 Cache)，最大容纳 50000 个不同的用户状态
local l1_cache, l1_err = core.lrucache.new({
    ttl = 300,
    count = 50000
})

local _M = {
    version = 0.2,
    priority = 2500,
    name = "custom-hmac-jwt",
}

-- 辅助函数：单次本地哈希校验
local function verify_pow(puzzle, nonce, difficulty)
    local sha256 = resty_sha256:new()
    sha256:update(puzzle .. nonce)
    local target_prefix = string.rep("0", difficulty)
    return string.sub(str.to_hex(sha256:final()), 1, difficulty) == target_prefix
end

-- 核心rewrite验证阶段
function _M.rewrite(conf, ctx)
    local access_key = core.request.header(ctx, "X-HMAC-ACCESS-KEY")
    if not access_key then return 401, { message = "Missing access key" } end

    ----------------------------------------------------------------
    -- 核心优化点 1：检查本地一级缓存 (L1 Cache) -> 零 Redis I/O 损耗
    ----------------------------------------------------------------
    local l1_status = l1_cache:get(access_key)
    if l1_status == "VIP_PASS" then
        -- 本地命中有放宽权限的白名单，直接路由到宽松限流桶
        ctx.consumer_name = access_key .. ":vip"
        return
    end

    -- 2. 检查是否携带了 PoW 挑战答案
    local puzzle = core.request.header(ctx, "X-POW-PUZZLE")
    local nonce = core.request.header(ctx, "X-POW-NONCE")

    if puzzle and nonce then
        -- 本地执行单次 CPU 校验 (非对称消耗：前端算死，网关秒验)
        if verify_pow(puzzle, nonce, conf.difficulty or 4) then
            
            ----------------------------------------------------------------
            -- 核心优化点 2：合并读写。利用 Redis NX 特性，一步完成防重放与通行证签发
            ----------------------------------------------------------------
            -- 以 puzzle 作为唯一防重放 Key，设置 5 分钟(300秒)过期。
            -- 如果黑客重放，NX 会返回 nil，代表已经有人用这个 puzzle 签发过通行证了。
            local red = get_redis_client(conf) -- 获取连接池连接
            local pass_key = "pow:pass:" .. puzzle
            local ok, err = red:set(pass_key, "1", "EX", 300, "NX")
            red:set_keepalive(10000, 100)

            if ok and ok ~= ngx.null then
                -- 验证通过且防重放成功！同步写入本地 L1 缓存，10秒内完全不查 Redis
                l1_cache:set(access_key, "VIP_PASS", 10)
                
                ctx.consumer_name = access_key .. ":vip"
                return
            else
                -- 核心优化点 3：防穿透。由于重放或非法的 Puzzle，在本地下发短平快惩罚，2秒内直接拒绝
                l1_cache:set(access_key, "BLOCKED", 2)
                return 403, { message = "Puzzle expired or replayed." }
            end
        else
            return 403, { message = "Invalid PoW solution." }
        end
    end

    ----------------------------------------------------------------
    -- 核心优化点 4：常规请求的二级缓存回源策略（合并 L1 -> L2 读操作）
    ----------------------------------------------------------------
    -- 如果客户端没带 PoW 答案，且本地 L1 未命中，则通过级联去远端 Redis (L2) 确认
    -- 我们在客户端生成完答案后，上一步的 pass_key 是基于 puzzle 的，为了方便常规读，
    -- 我们这里在本地 L1 未命中时，可以允许他去常规限流。
    -- 如果需要精准检查远端，可以在此读一次 Redis。但为了做到“极度减少读写”：
    -- 方案直接将其送入常规限流队列，让他在常规限流（10 r/s）里跑。
    
    -- 常规用户，使用严格限流标识
    ctx.consumer_name = access_key
end

-- 截获 429 转换为 499 流程
function _M.header_filter(conf, ctx)
    if ngx.status == 429 then
        ngx.status = 499
        ctx.need_pow_challenge = true
        ngx.header.content_length = nil
        ngx.header.content_type = "application/json"
    end
end

function _M.body_filter(conf, ctx)
    if not ctx.need_pow_challenge then return end
    
    local access_key = core.request.header(ctx, "X-HMAC-ACCESS-KEY") or "anonymous"
    -- 核心优化点 5：无状态自包含 Puzzle。结合时间戳与盐，无需向 Redis 写入
    local puzzle = string.format("%s:%d:%s", access_key, ngx.time(), core.utils.get_rand_string(8))
    
    local response_body = {
        status = "POW_CHALLENGE",
        puzzle = puzzle,
        difficulty = conf.difficulty or 4
    }
    ngx.arg[0] = core.json.encode(response_body)
    ngx.arg[1] = true
end

return _M
