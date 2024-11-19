local hc = require("hc")

local timer_map = {}

hc.register_protocol({
    name = "timer",
    ty = hc.TY_TIMER,
    israw = true,
    ---@param msg LuaMsg
    dispatch = function(msg) 
        local timerid = msg:read_i64()
        local v = timer_map[timerid]
        timer_map[timerid] = nil
        if not v then
            return
        end
        if type(v) == "thread" then
            hc.co_resume(v, timerid)
        else
            v()
        end

        
    end,
})