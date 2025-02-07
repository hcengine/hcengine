---@type core
local core = require("engine.core")

---@class hc : core
local hc = require("hc.core")

local function _hc_http_incoming(id, req)
    hc.print("_hc_http_incoming now = %o, %o %o", id, req, req:is_http2())

    local response = Response.new();
    response:set_text(string.format("from lua!!!!!! %d", id))
    response:header_set("ok", "val")
    hc.send_response(id, response)
end 

_G["hc_http_incoming"] = _hc_http_incoming