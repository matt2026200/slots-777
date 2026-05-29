-- 步骤 A：在 header_filter 阶段截获状态码并修改为 499
function _M.header_filter(conf, ctx)
    -- 如果发现被下游的限流插件拦住了（变成了 429），且当前不是已经跳过限流的请求
    if ngx.status == 429 and not ctx.skip_limit_req then
        -- 强行将返回给前端的 HTTP 状态码篡改为 499
        ngx.status = 499
        -- 并在上下文中打上标记，告诉接下来的 body_filter 阶段：“该换汤了”
        ctx.need_pow_challenge = true
        
        -- 清除原本限流插件输出的 Content-Length，因为我们要重写内容长度
        ngx.header.content_length = nil
        ngx.header.content_type = "application/json; charset=utf-8"
    end
end

-- 步骤 B：在 body_filter 阶段重写响应体，下发谜题
function _M.body_filter(conf, ctx)
    if not ctx.need_pow_challenge then
        return
    end

    local access_key = core.request.header(ctx, "X-HMAC-ACCESS-KEY") or "anonymous"
    
    -- 动态生成一个独一无二的谜题盐
    local salt = core.utils.get_rand_string(16)
    local puzzle = string.format("%s:%d:%s", access_key, ngx.time(), salt)

    local response_body = {
        status = "POW_CHALLENGE",
        message = "Please solve the puzzle to unlock.",
        puzzle = puzzle,
        difficulty = conf.difficulty or 4
    }

    -- 将包装好的 JSON 写回给客户端
    ngx.arg[0] = core.json.encode(response_body)
    ngx.arg[1] = true -- 标记响应结束 (EOF)
end

