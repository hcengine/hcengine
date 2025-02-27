---@type core
local core = require("engine.core")
local _bind_http = core.bind_http
local _http_request = core.http_request

---@class hc : core
local hc = require("hc.core")

local callback_table = {}
local callback_request = {}

local function calc_http_id(id)
    local high = id >> 32;
    return high & 0xFF
end

--- 绑定HTTP服务器
---@param addr string
hc.bind_http = function(addr, callback)
    local http_id = hc.wait(_bind_http(addr))
    callback_table[http_id] = callback
    return http_id
end

---@param id integer
---@param req Request
local function _hc_http_incoming(id, req)
    local http_id = calc_http_id(id)
    local response = nil
    if callback_table[http_id] then
        response = callback_table[http_id](req)
    end
    if not response then
        hc.print("_hc_http_incoming now = %o, %o %o", id, req, req:is_http2())
        hc.print("now url = %o", req:url())
        req:set_url("http://127.0.0.1:1111")
        hc.print("after url = %o", req:url())
        ---@type Response
        response = Response.new();
        response:set_status_code(201)
        response:set_text(string.format("from lua!!!!!! %d", id))
        response:header_set("ok", "val")
    end
    hc.send_response(id, response)
end 


-- ---@param id integer
-- ---@param res Response | nil
-- ---@param err string | nil
-- local function _hc_http_return(id, res, err)
--     if callback_request[id] then
--         callback_request[id](res, err)
--         callback_request[id] = nil
--     end
-- end

---@param req Request
---@param option ClientOption | nil
---@return Response | nil, string | nil
hc.http_request = function(req, option, callback)
    return hc.wait(_http_request(req, option))
    -- local session = _http_request(req, option)
    -- if session ~= 0 then
    --     callback_request[session] = callback
    -- end
end


_G["hc_http_incoming"] = _hc_http_incoming
-- _G["hc_http_return"] = _hc_http_return


