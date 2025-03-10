local core = require("engine.core")

local co_create = coroutine.create
local co_running = coroutine.running
local co_yield = coroutine.yield
local co_resume = coroutine.resume
local co_close = coroutine.close
if not co_close then
    co_close = function() end
end

---@return integer, string
local _newservice = core.new_service
local _send = core.send
local _resp = core.resp

local session_id_coroutine = {}
local session_id_record = {}
local protocol = {}
local timer_routine = {}
local timer_profile_trace = {}

---@class hc : core
local hc = core

local function wrap_co_resume(co, ...)
    local ok, err = co_resume(co, ...)
    if not ok then
        err = tostring(err)
        co_close(co)
        hc.print("%s", hc.traceback(true))
        error(err)
    end
    return ok, err
end


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

--- @return string|string[]
hc.args = function()
    return hc.env("args")
end

--- @return integer, integer @ 第一个是正在运行的co数量, 第二个缓存的co数量
hc.coroutine_num = function()
    return co_num, #co_pool
end

---@return integer | boolean, string
hc.wait = function(session, is_repeat)
    -- print("wait session = ", session)
    if session then
        session_id_coroutine[session] = co_running()
        -- 重复接收的不检查超时
        if not is_repeat then
            session_id_record[session] = hc.now_ms()
        else
            session_id_record[session] = nil
        end
    end

    local a, b, c = co_yield()
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
            return ret
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
    ---@type LuaMsg
    local msg = p.pack(...)
    msg.sender = hc.id
    msg.receiver = receiver
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
    local ret = hc.unpack(msg)
    if type(ret) ~= "table" then
        print("未知的lua协议类型, 必须为数组")
        return
    end
    local name = table.remove(ret, 1)
    -- local name, value = table.unpack(ret)
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



---@param conf ServiceConf
---@return integer|boolean, string
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
hc.TY_ERROR = 8;
hc.TY_REDIS = 9;
hc.TY_HTTP_RES = 10;
hc.TY_HTTP_REQ = 11;
hc.TY_MYSQL = 12;

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
    auto_unpack = true,
    pack = function(val)
        local msg = LuaMsg.new()
        msg.ty = hc.TY_INTEGER
        msg:write_i64(val)
        return msg
    end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return { msg:read_i64(), msg:get_err() }
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "number",
    ty = hc.TY_NUMBER,
    auto_unpack = true,
    pack = function(val)
        local msg = LuaMsg.new()
        msg.ty = hc.TY_NUMBER
        msg:write_f64(val)
        return msg
    end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return { msg:read_f64(), msg:get_err() }
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "string",
    ty = hc.TY_STRING,
    auto_unpack = true,
    pack = function(val)
        local msg = LuaMsg.new()
        msg.ty = hc.TY_STRING
        msg:write_str(val)
        return msg
    end,
    --- @param msg LuaMsg
    unpack = function(msg)
        return { msg:read_str(), msg:get_err() }
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "lua",
    ty = hc.TY_ERROR,
    auto_unpack = true,
    pack = hc.pack,
    unpack = function(msg)
        return { nil, msg:get_err() }
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "redis",
    ty = hc.TY_REDIS,
    pack = hc.pack,
    unpack = function(msg)
        return msg:read_obj()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "mysql",
    ty = hc.TY_MYSQL,
    pack = hc.pack,
    unpack = function(msg)
        return msg:read_obj()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "http-res",
    ty = hc.TY_HTTP_RES,
    pack = hc.pack,
    unpack = function(msg)
        return msg:read_obj()
    end,
    dispatch = function() end,
})

hc.register_protocol({
    name = "http-req",
    ty = hc.TY_HTTP_REQ,
    pack = hc.pack,
    unpack = function(msg)
        return msg:read_obj()
    end,
    dispatch = function() end,
})

------protocol message ----------------------
---------------------------------------------

local delay_init_func = {}
hc.register_init = function(fn)
    table.insert(delay_init_func, fn)
end

hc.init = function()
    for _, fn in ipairs(delay_init_func) do
        fn()
    end
    delay_init_func = {}
end

hc.register_init(function()
    hc.timeout(4999, true, check_coroutine_timeout)
end)

return hc;
