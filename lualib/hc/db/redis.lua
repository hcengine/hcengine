local core = require("engine.core")
local _set_redis_url = core.set_redis_url
local _run_redis_command = core.run_redis_command

---@class hc : core
local hc = require("hc.core")

hc.redis_index = 0

hc.set_redis_index = function(index)
    hc.redis_index = index
end

hc.set_redis_url = function(url)
    hc.redis_index = _set_redis_url(url)
    return hc.redis_index
end

hc.run_redis_command = function(...)
    hc.print("run_redis_command args = %o", {...})
    return hc.run_redis_command_with_index(hc.redis_index, ...)
end

hc.run_redis_command_with_index = function(index, ...)
    return hc.wait(_run_redis_command(index, ...))
end

hc.run_subs_command = function(callback, ...)
    return hc.run_redis_command_with_index(hc.redis_index, callback, ...)
end

hc.run_subs_command_with_index = function(index, callback, ...)
    local session = _run_redis_command(index, "SUBSCRIBE", ...)
    hc.print("subs session = %o", session)
    while true do
        local val, ret = hc.wait(session, true)
        if ret then
            return
        end
        callback(val)
    end
end


hc.run_psubs_command = function(callback, ...)
    return hc.run_psubs_command_with_index(hc.redis_index, callback, ...)
end

hc.run_psubs_command_with_index = function(index, callback, ...)
    local session = _run_redis_command(index, "PSUBSCRIBE", ...)
    while true do
        local val, ret = hc.wait(session, true)
        if ret then
            return
        end
        callback(val)
    end
end