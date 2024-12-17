---@type core
local core = require("engine.core")

local _bind_listen = core.bind_listen

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

    data_listen_cb[id] = cb
end


local function _net_on_accept_dispath(id, new_id)
    hc.print("--------------- _net_on_accept_dispath = %o, %o", id, new_id)
    local callback = data_listen_cb[id]
    if not callback or not callback["cb"] then
        hc.print("未设置处理回调函数，关闭连接:%o", new_id)
        hc.close_socket(new_id, "not callback");
        return
    end
    -- callback["on_accept"]
end

local function _net_on_close_dispath(id, new_id)
    hc.print("--------------- _net_on_close_dispath = %o, %o", id, new_id)
    local callback = data_listen_cb[id]
    if not callback or not callback["cb"] then
        -- hc.close_socket(new_id, "already close");
        return
    end
    -- callback["on_accept"]
end

_G["hc_accept_conn"] = _net_on_accept_dispath
_G["hc_close_conn"] = _net_on_close_dispath