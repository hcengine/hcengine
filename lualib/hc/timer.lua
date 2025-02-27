---@type core
local core = require("engine.core")

---@class hc : core
local hc = require("hc.core")

local _timeout = core.timeout
local _del_timer = core.del_timer

---------------------------------------------
------timer oper       ----------------------
---------------------------------------------

local timer_cache = {};
setmetatable(timer_cache, { __mode = "k" });

hc.register_protocol({
    name = "timer",
    ty = hc.TY_TIMER,
    israw = true,
    ---@param msg LuaMsg
    dispatch = function(msg)
        print("timer dissssssssssssssssssss", msg)
        local timerid = msg:read_i64()
        local is_repeat = msg:read_bool()
        local v = timer_cache[timerid]
        print("id = ", timerid, " ispreat = ", is_repeat, " call back = ", v)
        if not is_repeat then
            timer_cache[timerid] = nil
        end
        if not v then
            return
        end
        if type(v) == "thread" then
            hc.co_resume(v, timerid, msg:get_err())
        else
            hc.async(v)
        end
    end,
})

--- 超时消息
--- @param mills integer
--- @param is_repeat boolean
--- @return integer
hc.timeout = function(mills, is_repeat, fn)
    local timer_id = _timeout(mills, is_repeat or false)
    timer_cache[timer_id] = fn
    return timer_id
end

--- 超时消息
--- @param mills integer
--- @return boolean, string | nil
hc.sleep = function(mills)
    local timer_id = _timeout(mills, false)
    timer_cache[timer_id] = coroutine.running()
    local id, reason = coroutine.yield()
    if id ~= timer_id then
        timer_cache[timer_id] = nil
        return false, reason
    end
    return true
end

--- 超时消息
--- @param timer_id integer
hc.del_timer = function(timer_id)
    _del_timer(timer_id)
end

hc.get_timer = function(time_id)
    return timer_cache[time_id];
end

---------------------------------------------
------timer oper       ----------------------
---------------------------------------------

