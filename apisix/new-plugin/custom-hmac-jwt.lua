local core     = require("apisix.core")
local hmac     = require("apisix.plugins.hmac-auth") -- 复用官方的签名计算逻辑
local resty_redis = require("resty.redis")

local schema = {
    type = "object",
    properties = {
        redis_host = {type = "string", default = "127.0.0.1"},
        redis_port = {type = "integer", default = 6379},
        redis_timeout = {type = "integer", default = 1000}, -- 毫秒
    }
}

local plugin_name = "custom-hmac-jwt"

local _M = {
    version = 0.1,
    priority = 2500, -- 优先级调高，在限流(limit-count)之前执行
    name = plugin_name,
    schema = schema,
}

-- 辅助函数：从 Redis 动态获取 Secret Key
local function get_secret_from_redis(conf, access_key)
    local red = resty_redis:new()
    red:set_timeout(conf.redis_timeout)
    local ok, err = red:connect(conf.redis_host, conf.redis_port)
    if not ok then
        return nil, "failed to connect to redis: " .. err
    end

    -- 假设存储结构为 String，Key 格式为 "hmac:secret:[access_key]"
    local res, err = red:get("hmac:secret:" .. access_key)
    -- 放回连接池提高性能
    red:set_keepalive(10000, 100)

    if not res or res == ngx.null then
        return nil, "secret not found"
    end
    return res
end

function _M.check_schema(conf)
    return core.schema.check(schema, conf)
end

function _M.rewrite(conf, ctx)
    -- 1. 从 HTTP Header 提取客户端传过来的 Access Key (这里可以映射为 JWT_ID)
    local access_key = core.request.header(ctx, "X-HMAC-ACCESS-KEY")
    if not access_key then
        return 401, { message = "Missing access key" }
    end

    -- 2. 动态去 Redis 查出对应的 Secret Key
    local secret_key, err = get_secret_from_redis(conf, access_key)
    if err or not secret_key then
        core.log.error("Fetch secret failed: ", err)
        return 401, { message = "Invalid or expired session" }
    end

    -- 3. 构造一个临时的 consumer 配置，瞒过官方的校验逻辑
    -- 因为官方的 hmac 插件校验函数需要一个包含 secret_key 的结构
    local mock_plugin_conf = {
        access_key = access_key,
        secret_key = secret_key,
        algorithm = core.request.header(ctx, "X-HMAC-ALGORITHM") or "hmac-sha256"
    }

    -- 4. 调用官方的 hmac 验证逻辑 (包含防止重放攻击的时间戳校验)
    -- 注意：需要参考 APISIX 源码中 hmac-auth 的具体签名比对方法，这里简写逻辑
    local is_valid, verify_err = hmac.do_verification(ctx, mock_plugin_conf) 
    if not is_valid then
        return 401, { message = "HMAC signature verification failed: " .. (verify_err or "") }
    end

    -- 5. 校验通过！将用户 ID 注入到上下文，供后面的限流插件(limit-count)做针对性限流
    ctx.consumer_name = access_key 
end

return _M
