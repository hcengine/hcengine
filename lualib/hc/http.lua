---@type core
local core = require("engine.core")

---@class hc : core
local hc = require("hc.core")

local function _hc_http_incoming(id, req)
    hc.print("_hc_http_incoming now = %o, %o", id, req)
end 

_G["hc_http_incoming"] = _hc_http_incoming