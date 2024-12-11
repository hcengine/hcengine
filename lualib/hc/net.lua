---@type core
local core = require("engine.core")

---@class hc : core
local hc = require("hc.core")

hc.listen = function(method, url, settings, on_accept, on_close)
    local id = hc.wait(hc.bind_listen(method, url))
    if not id then
        hc.exit(-1)
        return
    end
    
end