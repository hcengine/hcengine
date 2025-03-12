local core = require("engine.core")
local _set_redis_url = core.set_redis_url
local _run_redis_command = core.run_redis_command
local _get_redis_keep = core.get_redis_keep
local _del_redis_keep = core.del_redis_keep

---@class hc : core
local hc = require("hc.core")

---@class redis
local redis = {}

redis.index = 0
redis.keep = 0

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

---设置默认redis的url映射索引
function redis:get_redis_keep()
    local keep, err = hc.wait(_get_redis_keep(self.index))
    if (keep or 0) ~= 0 then
        self.keep = keep
    end
    return keep, err
end

---设置默认redis的url映射索引
function redis:det_redis_keep()
    _del_redis_keep(self.index, self.keep or 0)
end

---运行redis命令
function redis:run_redis_command(...)
    hc.print("run_redis_command = %o %o", self.index, self.keep)
    return hc.wait(_run_redis_command(self.index, self.keep, ...))
end

---订阅消息
function redis:run_subs_command(callback, ...)
    local session = _run_redis_command(self.index, self.keep, "SUBSCRIBE", ...)
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
    local session = _run_redis_command(self.index, self.keep, "PSUBSCRIBE", ...)
    while true do
        local val, ret = hc.wait(session, true)
        if ret then
            return
        end
        callback(val)
    end
end

local function redis_default(r, key)
    hc.print("new index!!!! %o", key)
    local k = string.upper(key)
    local func = function(val, ...)
        hc.print("val = %o", val)
        return val:run_redis_command(k, ...)
    end
    rawset(redis, key, func)
    return func
end

local function redis_index(r, key)
    if redis[key] then
        return redis[key]
    end
    return redis_default(r, key)
end

---@return redis
function redis:build_connect(table)
    setmetatable(table, { __index = redis_index })
    return table
end

local meta = {
    __index = redis_default,
}
setmetatable(redis, meta)

return redis