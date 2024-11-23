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
            v()
        end
    end,
})

--- 超时消息
--- @param interval integer
--- @param is_repeat boolean
--- @return integer
hc.timeout = function(interval, is_repeat, fn)
    local timer_id = _timeout(interval, is_repeat or false)
    timer_cache[timer_id] = fn

    return timer_id
end

--- 超时消息
--- @param interval integer
--- @return boolean, string | nil
hc.sleep = function(interval)
    local timer_id = _timeout(interval, false)
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

function get_timer(time_id)
    return timer_cache[time_id];
end

function get_all_timer()
    return timer_cache;
end

function get_timer_count()
    return #timer_cache;
end
---------------------------------------------
------timer oper       ----------------------
---------------------------------------------

