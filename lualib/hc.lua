local core = require("hc.core")

local co_create = coroutine.create
local co_running = coroutine.running
local co_yield = coroutine.yield
local co_resume = coroutine.resume
local co_close = coroutine.close
if not co_close then
    co_close = function() end
end

local _newservice = core.new_service

local session_id_coroutine = {}
local protocol = {}
local session_watcher = {}
local timer_routine = {}
local timer_profile_trace = {}

local function wrap_co_resume(co, ...)
    local ok, err = co_resume(co, ...)
    if not ok then
        err = tostring(err)
        co_close(co)
        error(err)
    end
    return ok, err
end

---@class hc : core
local hc = core



local co_num = 0

local co_pool = setmetatable({}, { __mode = "kv" })

local function invoke(co, fn, ...)
    co_num = co_num + 1
    fn(...)
    co_num = co_num - 1
    co_pool[#co_pool + 1] = co
end

local function routine(fn, ...)
    local co = co_running()
    invoke(co, fn, ...)
    while true do
        invoke(co, co_yield())
    end
end

hc.bootstrap_id = 1

hc.async = function(fn, ...)
    local co = table.remove(co_pool) or co_create(routine)
    wrap_co_resume(co, fn, ...)
    return co
end

hc.wait = function(session, receiver)
    if session then
        session_id_coroutine[session] = co_running()
        if receiver then
            session_watcher[session] = receiver
        end
    else
        if type(receiver) == "string" then -- receiver is error message
            return false, receiver
        end
    end

    local a, b, c = co_yield()
    print("xxx ", a, b, c)
    if a then
        -- LuaMsg
        return protocol[a.ty].unpack(a)
    else
        -- false, "BREAK", {...}
        if session then
            session_id_coroutine[session] = nil
        end

        if c then
            return table.unpack(c)
        else
            return a, b --- false, "BREAK"
        end
    end
end

---@param conf ServiceConf
hc.new_service = function(conf)
    return hc.wait(_newservice(conf))
end

---------------------------------------------
------protocol message ----------------------

hc.TY_UNKNOWN = 0;
hc.TY_INTEGER = 1;
hc.TY_NUMBER = 2;
hc.TY_STRING = 3;
hc.TY_LUA = 4;


hc.register_protocol = function(t)
    local ty = t.ty
    if protocol[ty] then
        print("重复注册协议:", ty)
    end
    protocol[ty] = t
    protocol[t.name] = t
end

hc.register_protocol({
    name = "lua",
    ty = hc.TY_LUA,
    pack = function() end,
    unpack = function() end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "integer",
    ty = hc.TY_INTEGER,
    pack = function(...) return ... end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return msg:read_i64(), msg:get_err()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "number",
    ty = hc.TY_NUMBER,
    pack = function(...) return ... end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return msg:read_f64(), msg:get_err()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "string",
    ty = hc.TY_STRING,
    pack = function(...) return ... end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return msg:read_str(), msg:get_err()
    end,
    dispatch = function() end,
})
------protocol message ----------------------
---------------------------------------------

---@param msg LuaMsg
local function _wrap_dispath(msg)
    local p = protocol[msg.ty]
    if not p then
        error(string.format("handle unknown ptype: %s. sender %u", msg.ty, msg.sender))
    end

    local session = msg.sessionid
    if session > 0 then
        session_watcher[session] = nil
        local co = session_id_coroutine[session]
        if co then
            session_id_coroutine[session] = nil
            wrap_co_resume(co, msg)
            return
        end

        if not co then
            error(string.format("%s: response [%u] can not find co.", hc.name, session))
        end
    else
        local dispatch = p.dispatch
        if not dispatch then
            error(string.format("[%s] dispatch ptype [%u] is nil", hc.name, p.ptype))
            return
        end

        if not p.israw then
            local co = table.remove(co_pool) or co_create(routine)
            if not p.unpack then
                error(string.format("ptype %s has no unpack function.", p.ptype))
            end
            wrap_co_resume(co, dispatch, msg)
        else
            dispatch(msg)
        end
    end
end

---@param msg LuaMsg
local function _dispath(msg)
    _wrap_dispath(msg)
    LuaMsg.del(msg)
end

--- 关闭lua接口, 清理lua资源, 将不能再调用任何数据
local function _stop_world()
    print("stop_world:%d", hc.id)
end

--- 消息分配器
_G["hc_msg_dispath"] = _dispath
_G["stop_world"] = _stop_world

print("cccccccccccc?????")

return hc;
