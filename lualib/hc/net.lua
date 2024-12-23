---@type core
local core = require("engine.core")

local _bind_listen = core.bind_listen
local _send_msg = core.send_msg

local data_listen_cb = {}

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
--- @param method string
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
        on_accept(new_id, callback)
    end
    data_listen_cb[new_id] = callback
end

local function _net_on_close_dispath(id, new_id)
    hc.print("--------------- _net_on_close_dispath = %o, %o", id, new_id)
    local callback = data_listen_cb[id]
    if not callback then
        -- hc.close_socket(new_id, "already close");
        return
    end
    -- callback["on_accept"]
end

--- @param id integer
--- @param msg NetMsg
local function _net_on_msg(id, msg)
    hc.print("--------------- _net_on_msg = %o, t = %o msg = %o", id, msg:get_type(), msg)
    local t = msg:get_type()
    if t == hc.NET_PING then
        hc.print("ping!!!!!!!!!!!!!!!!! %o %o", msg, msg:get_lstring())
        local pong = NetMsg.pack_pong(msg:get_lstring())
        hc.send_msg(id, pong)
    end
    hc.print("string = %o", msg:get_string())
    hc.print("NetMsg = %o", NetMsg)
    local send = NetMsg.pack_text(string.format("from lua %s", msg:get_string()))
    local meta = getmetatable(send)
    hc.print("send = %o meta = %o", send, meta)
    NetMsg.del(msg)
    local meta = getmetatable(send)
    hc.print("send1 = %o meta = %o", send, meta)
    hc.print("del end!!!!!!!!!!!!!!!")
    hc.send_msg(id, send)
    
    local send = NetMsg.pack_binary(string.format("from lua %s", msg:get_string()))
    hc.send_msg(id, send)
    hc.print("send end!!!!!!!!!!!!!!!")
    local callback = data_listen_cb[id]
    if not callback then
        -- hc.close_socket(new_id, "already close");
        return
    end
    -- callback["on_accept"]
end


_G["hc_net_accept_conn"] = _net_on_accept_dispath
_G["hc_net_close_conn"] = _net_on_close_dispath
_G["hc_net_msg"] = _net_on_msg