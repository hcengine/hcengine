---@type core
local core = require("engine.core")

local _bind_listen = core.bind_listen
local _connect = core.connect
local _send_msg = core.send_msg

local data_listen_cb = {}
local ready_connect_ids = {}

---@class hc : core
local hc = require("hc.core")

--- 绑定连接
--- @param method string
--- @param url string
--- @param settings table | nil
--- @return integer
hc.bind_listen = function(method, url, settings)
    return _bind_listen(method, url, settings or nil)
end

--- 绑定连接
--- @param method string | "'ws'" | "'tcp'" | "'kcp'"
--- @param url string
--- @param settings table | nil
--- @param cb net_cb 
hc.listen = function(method, url, settings, cb)
    local id = hc.wait(hc.bind_listen(method, url, settings))
    if not id then
        hc.exit(-1)
        return
    end

    hc.print("id === %o", id)
    data_listen_cb[id] = cb
    if cb["on_open"] then
        cb["on_open"](id)
    end
    return id
end

--- 发起连接
--- @param method string | "'ws'" | "'tcp'" | "'kcp'"
--- @param url string
--- @param settings table | nil
--- @param cb net_cb 
hc.connect = function(method, url, settings, cb)
    local id = hc.wait(_connect(method, url, settings))
    hc.print("hc.connect !!!!!!!!!!! = %o", id)
    if not id then
        return false
    end

    ready_connect_ids[id] = true
    hc.print("connect id === %o", id)
    data_listen_cb[id] = cb
    return true, id
end

--- 发送消息
--- @param id integer
--- @param msg NetMsg
hc.send_msg = function(id, msg)
    if not ready_connect_ids[id] then
        hc.print("该id(%o)未准备就绪, 请等待id就绪后发送!", id)
        return
    end
    _send_msg(id, msg)
end

local function _net_on_accept_dispath(id, new_id, socket_addr)
    hc.print("--------------- _net_on_accept_dispath = %o, %o, %o, %o", id, new_id, data_listen_cb[id], socket_addr)
    local callback = data_listen_cb[id]
    if not callback then
        hc.print("未设置处理回调函数，关闭连接:%o", new_id)
        hc.close_socket(new_id, "not callback");
        return
    end
    local on_accept = callback["on_accept"]
    if on_accept then
        local cb = on_accept(new_id)
        if cb then
            data_listen_cb[new_id] = cb
        else
            data_listen_cb[new_id] = callback
        end
    end
end

local function _net_on_close_dispath(id, new_id)
    hc.print("--------------- _net_on_close_dispath = %o, %o", id, new_id)
    local callback = data_listen_cb[new_id]
    if not callback then
        return
    end
    local on_close = callback["on_close"]
    if on_close then
        on_close(new_id)
    end
    data_listen_cb[new_id] = nil
    ready_connect_ids[new_id] = nil
end

local function _net_on_open_dispath(id)
    hc.print("--------------- _net_on_open_dispath = %o, %o", id)
    local callback = data_listen_cb[id]
    if not callback then
        return
    end
    local on_open = callback["on_open"]
    if on_open then
        on_open(id)
    end
    ready_connect_ids[id] = true
end


local function _wrap_net_on_msg(id, msg)
    local callback = data_listen_cb[id]
    if not callback then
        hc.close_socket(id, "already close");
        return
    end

    local t = msg:get_type()
    if t == hc.NET_PING then
        hc.print("ping!!!!!!!!!!!!!!!!! %o %o", msg, msg:get_lstring())
        local ping_cb = callback["on_ping"]
        if ping_cb then
            ping_cb(id, msg)
        else
            local pong = NetMsg.pack_pong(msg:get_lstring())
            hc.send_msg(id, pong)
        end
        return
    elseif t == hc.NET_PONG then
        hc.print("pong!!!!!!!!!!!!!!!!! %o %o", msg, msg:get_lstring())
        local pong_cb = callback["on_pong"]
        if pong_cb then
            pong_cb(id, msg)
        else
            
        end
        return
    elseif t == hc.NET_TEXT or t == hc.NET_BINARY then
        hc.print("msg !!!!!!!!!!!!!!!!! %o %o", msg, msg:get_lstring())
        local msg_cb = callback["on_msg"]
        if msg_cb then
            return msg_cb(id, msg)
        else
            hc.print("not callback %o on_msg!!!!", id)
        end
        return
    else
        hc.print("--------------- _net_on_msg = %o, t = %o msg = %o", id, hc.NET_NAMES[msg:get_type()], msg)
    end
end

--- @param id integer
--- @param msg NetMsg
local function _net_on_msg(id, msg)
    local is_move = _wrap_net_on_msg(id, msg)
    if not is_move then
        NetMsg.del(msg)
    end
end


_G["hc_net_accept_conn"] = _net_on_accept_dispath
_G["hc_net_open_conn"] = _net_on_open_dispath
_G["hc_net_close_conn"] = _net_on_close_dispath
_G["hc_net_msg"] = _net_on_msg