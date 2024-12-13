---@type core
local core = require("engine.core")

local _bind_listen = core.bind_listen

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
--- @param on_accept function | nil
--- @param on_close function | nil
--- @param cb net_cb 
hc.listen = function(method, url, settings, on_accept, on_close, cb)
    local id = hc.wait(hc.bind_listen(method, url, settings))
    if not id then
        hc.exit(-1)
        return
    end

end