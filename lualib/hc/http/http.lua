---@type core
local core = require("engine.core")
local _bind_http = core.bind_http
local _http_request = core.http_request

---@class hc : core
local hc = require("hc.core")

local router_table = {}

local function calc_http_id(id)
    local high = id >> 32;
    return high & 0xFF
end

--- 绑定HTTP服务器
---@param addr string
hc.bind_http = function(addr, callback)
    local http_id = hc.wait(_bind_http(addr))
    router_table[http_id] = callback
    return http_id
end

---@param id integer
---@param req Request
local function _hc_http_incoming(id, req)
    local http_id = calc_http_id(id)
    ---@type Response
    local res = Response.new()
    if type(router_table[http_id]) == "function"  then
        res = router_table[http_id](req, res)
    elseif type(router_table[http_id]) == "table" then

    end
    if not res then
        res:set_status_code(502)
        res:set_body("server internal error")
    end
    hc.send_response(id, res)
end


---@param req Request
---@param option ClientOption | nil
---@return Response | nil, string | nil
hc.http_request = function(req, option)
    ---@diagnostic disable-next-line: return-type-mismatch
    return hc.wait(_http_request(req, option))
end

hc.http_get = function(url, option)
    local req = Request.new()
    req:set_method("GET")
    req:set_url(url);
    return hc.wait(_http_request(req, option))
end

hc.http_post = function(url, body, option)
    local req = Request.new()
    req:set_method("POST")
    req:set_url(url);
    req:set_body(body)
    return hc.wait(_http_request(req, option))
end

hc.http_post_form = function(url, form, option)
    local req = Request.new()
    req:set_method("POST")
    req:set_url(url);
    req:header_set("Content-Type", "application/x-www-form-urlencoded")
    local body = hc.create_query_string(form)
    req:set_body(body)
    return hc.wait(_http_request(req, option))
end

hc.http_post_json = function(url, json, option)
    local req = Request.new()
    req:set_method("POST")
    req:set_url(url);
    req:header_set("Content-Type", "application/json")
    local body = hc.encode_json(json)
    req:set_body(body)
    return hc.wait(_http_request(req, option))
end

_G["hc_http_incoming"] = _hc_http_incoming
