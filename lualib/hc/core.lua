local core = require("engine.core")

local co_create = coroutine.create
local co_running = coroutine.running
local co_yield = coroutine.yield
local co_resume = coroutine.resume
local co_close = coroutine.close
if not co_close then
    co_close = function() end
end

local _newservice = core.new_service
local _send = core.send
local _resp = core.resp

local session_id_coroutine = {}
local session_id_record = {}
local protocol = {}
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

local function check_coroutine_timeout()
    local now = hc.now_ms()
    print("check_coroutine_timeout", now)
    for id, value in pairs(session_id_record) do
        if now - value > 10000 then
            local co = session_id_coroutine[id]
            session_id_coroutine[id] = nil
            session_id_record[id] = nil
            if co then
                co_resume(co, false, "TIMEOUT")
            end
        end
    end
end

hc.pack = Protocol.lua_pack
--- @param msg LuaMsg
hc.unpack = function(msg)
    return Protocol.lua_unpack(msg)
end

hc.bootstrap_id = 1

hc.async = function(fn, ...)
    local co = table.remove(co_pool) or co_create(routine)
    wrap_co_resume(co, fn, ...)
    return co
end

hc.co_resume = wrap_co_resume

---@return integer | boolean, string
hc.wait = function(session, receiver)
    print("wait session = ", session)
    if session then
        session_id_coroutine[session] = co_running()
        session_id_record[session] = hc.now_ms()
    else
        if type(receiver) ~= "number" then -- receiver is error message
            return false, receiver
        end
    end

    print("hc.wait ", session, receiver)
    local a, b, c = co_yield()
    print("xxx ", a, b, c)
    if a then
        local proto = protocol[a.ty]
        if not proto then
            return false, "unknow proto"
        end
        local ret = proto.unpack(a)
        if proto.auto_unpack then
            if type(ret) ~= "table" then
                return false, "proto error"
            end
            -- LuaMsg
            return table.unpack(ret)
        else
            -- LuaMsg
            return ret, receiver
        end
    else
        -- false, "BREAK", {...}
        -- or false, "TIMEOUT"
        if session then
            session_id_coroutine[session] = nil
            session_id_record[session] = nil
        end

        if c then
            return table.unpack(c)
        else
            return a, b --- false, "BREAK"
        end
    end
end

---@param ty integer | string msg type
---@param receiver integer service_id
---@return ...
hc.call = function(ty, receiver, ...)
    local p = protocol[ty]
    if not p then
        error(string.format("未知的协议类型:%s", ty))
    end
    if receiver == 0 then
        error("call receiver == 0")
    end
    print("zzzzzzzzzzzzzzz")
    ---@type LuaMsg
    local msg = p.pack(...)
    msg.sender = hc.id
    msg.receiver = receiver
    print("2222222222222")
    return hc.wait(_send(msg))
end


---@param ty integer msg type
---@param receiver integer service_id
---@return ...
hc.response = function(ty, receiver, sessionid, ...)
    local p = protocol[ty]
    if not p then
        error(string.format("未知的协议类型:%s", ty))
    end
    if receiver == 0 then
        error("call receiver == 0")
    end
    ---@type LuaMsg
    local msg = p.pack(...)
    msg.receiver = receiver
    msg.sessionid = sessionid
    return _resp(msg)
end

---@param ty string
---@param fn fun(msg: LuaMsg)
hc.dispatch = function(ty, fn)
    local p = protocol[ty]
    if fn then
        p.dispatch = fn
    end
end


---@param msg LuaMsg
local function _wrap_response(msg)
    local p = protocol[msg.ty]
    if not p then
        error(string.format("handle unknown ty: %s. sender %u", msg.ty, msg.sender))
    end

    local session = msg.sessionid
    if session > 0 then
        local co, _ = table.unpack(session_id_coroutine[session] or {})
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
            error(string.format("[%s] dispatch ty [%u] is nil", hc.name, p.ty))
            return
        end

        if not p.israw then
            local co = table.remove(co_pool) or co_create(routine)
            if not p.unpack then
                error(string.format("ty %s has no unpack function.", p.ty))
            end
            wrap_co_resume(co, dispatch, msg)
        else
            dispatch(msg)
        end
    end
end

---@param msg LuaMsg
local function _response(msg)
    _wrap_response(msg)
    LuaMsg.del(msg)
end

--- comment
--- @param msg LuaMsg
local function _wrap_lua_msg_dispath(msg)
    print("msg ============ ", msg)
    local ret = hc.unpack(msg)
    if type(ret) ~= "table" then
        print("未知的lua协议类型, 必须为数组")
        return
    end
    print("msg ============ ret ", ret)
    local name = table.remove(ret, 1)
    -- local name, value = table.unpack(ret)
    print("msg ============ zzzzzzzzzzzzzzzzzz ", msg)
    if type(name) ~= "string" then
        print("未知的协议消息或者协议参数")
        return
    end
    local func = _G[name]
    if type(func) ~= "function" then
        print(string.format("未找到函数:%s,无法处理消息", name))
        return
    end

    local sender = msg.sender
    local session = msg.sessionid

    print("_wrap_lua_msg_dispath ============ ", sender, session)
    if session ~= 0 and sender ~= 0 then
        hc.response(msg.ty, sender, session, func(table.unpack(ret)))
    else
        func(table.unpack(ret))
    end
end

local function _lua_msg_dispath(msg)
    _wrap_lua_msg_dispath(msg)
    LuaMsg.del(msg)
end

--- 关闭lua接口, 清理lua资源, 将不能再调用任何数据
local function _stop_world()
    print("stop_world:%d", hc.id)
end

--- 消息分配器
_G["hc_msg_call"] = _lua_msg_dispath
_G["hc_msg_resp"] = _response
_G["stop_world"] = _stop_world

print("cccccccccccc?????")


---@param conf ServiceConf
---@return integer, string
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
hc.TY_LUA_MSG = 5;
hc.TY_NET = 6;
hc.TY_TIMER = 7;

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
    auto_unpack = true,
    pack = hc.pack,
    unpack = hc.unpack,
    dispatch = function() end,
})

hc.register_protocol({
    name = "lua_msg",
    ty = hc.TY_LUA_MSG,
    auto_unpack = true,
    pack = hc.pack,
    unpack = hc.unpack,
    dispatch = _lua_msg_dispath,
})

hc.register_protocol({
    name = "net",
    ty = hc.TY_NET,
    pack = hc.pack,
    unpack = hc.unpack,
    dispatch = function() end,
})


hc.register_protocol({
    name = "integer",
    ty = hc.TY_INTEGER,
    pack = function(val)
        local msg = LuaMsg.new()
        msg.ty = hc.TY_INTEGER
        msg:write_u64(val)
        return msg
    end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return msg:read_i64(), msg:get_err()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "number",
    ty = hc.TY_NUMBER,
    pack = function(val)
        local msg = LuaMsg.new()
        msg.ty = hc.TY_NUMBER
        msg:write_f32(val)
        return msg
    end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return msg:read_f64(), msg:get_err()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "string",
    ty = hc.TY_STRING,
    pack = function(val) 
        local msg = LuaMsg.new()
        msg.ty = hc.TY_STRING
        msg:write_str(val)
        return msg
    end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return msg:read_str(), msg:get_err()
    end,
    dispatch = function() end,
})
------protocol message ----------------------
---------------------------------------------

hc.init = function()
    hc.timeout(1000, true, check_coroutine_timeout)
end

return hc;
