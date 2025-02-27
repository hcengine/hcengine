local core = require("engine.core")
local _set_redis_url = core.set_redis_url
local _run_redis_command = core.run_redis_command

---@class hc : core
local hc = require("hc.core")

local redis = {}

redis.index = 0

---设置默认redis的索引
---@param index integer
function redis:set_redis_index(index)
    self.index = index
end

---设置默认redis的url映射索引
---@param url string
function redis:set_redis_url(url)
    self.index = _set_redis_url(url)
    return self.index
end

---运行redis命令
function redis:run_redis_command(...)
    return redis.run_redis_command_with_index(redis.index, ...)
end

---运行redis根据索引
function redis.run_redis_command_with_index(index, ...)
    hc.print("run_redis_command_with_index args = %o", {index, ...})
    return hc.wait(_run_redis_command(index, ...))
end

---订阅消息
function redis:run_subs_command(callback, ...)
    return redis.run_redis_command_with_index(self.index, callback, ...)
end

---订阅消息根据索引
function redis.run_subs_command_with_index(index, callback, ...)
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


---订阅批消息
function redis:run_psubs_command(callback, ...)
    return redis.run_psubs_command_with_index(self.index, callback, ...)
end

---订阅批消息根据索引
function redis.run_psubs_command_with_index(index, callback, ...)
    local session = _run_redis_command(index, "PSUBSCRIBE", ...)
    while true do
        local val, ret = hc.wait(session, true)
        if ret then
            return
        end
        callback(val)
    end
end

---redis的设置key val
function redis:set(key, val)
    return self:run_redis_command("SET", key, val)
end

---redis的设置key val
function redis.set_with_index(index, key, val)
    return redis.run_redis_command_with_index(index, "SET", key, val)
end

-- ---redis的设置key val
-- function redis:get(key)
--     return self:run_redis_command("GET", key)
-- end

-- ---redis的设置key val
-- function redis.get_with_index(index, key)
--     return redis.run_redis_command_with_index(index, "GET", key)
-- end

-- ---删除key
-- function redis:del(key)
--     return self:run_redis_command("DEL", key)
-- end

-- ---删除key
-- function redis.del_with_index(index, key)
--     return redis.run_redis_command_with_index(index, "DEL", key)
-- end


local meta = {
    __index = function(r, key)
        hc.print("new index!!!! %o", key)
        local s = string.find(key, "_with_index")
        if s then
            local command = string.sub(key, 1, s - 1)
            hc.print("command!!!! %o", command)
            local k = string.upper(command)
            local func = function(index, ...)
                return r.run_redis_command_with_index(index, k, ...)
            end
            rawset(r, key, func)
            return func

        else
            local k = string.upper(key)
            local func = function(val, ...)
                return val:run_redis_command(k, ...)
            end
            rawset(r, key, func)
            return func
        end
    end,
    -- __newindex = function(r, key, v)
    --     hc.print("new index!!!! %o v = %o", key, v)
    --     local k = string.upper(key)
    --     rawset(r, key, function(val, ...)

    --         return val:run_redis_command(k, ...)
    --     end)
    --     return r[key]
    -- end
}
setmetatable(redis, meta)

return redis