local hc = require("lualib.hc")

local default_value = "default return"
local test_value = "test return"
local test_reg = "test reg"

hc.async(function()
    ---@type Router
    local router = hc.Router.new(function(req, res)
        res:set_status_code(201)
        res:set_body(default_value)
        res:header_set("ok", "val")
        return res
    end)

    hc.print("is valid = %o", router:is_vaild())

    router:on("/test", function(req, res)
        res:set_status_code(200)
        res:set_body(test_value)
        res:header_set("ok", "val")
        return res
    end)

    router:on_reg("/test(%w+)", function(req, res)
        res:set_status_code(200)
        res:set_body(test_reg)
        res:header_set("ok", "val")
        return res
    end)

    --@param req Request
    hc.bind_http("0.0.0.0:8082", router)
    -- --@type Request
    hc.timeout(1000, false, function()
        local req = Request.new()
        req:set_url("http://127.0.0.1:8082/");

        local res, err = hc.http_request(req, nil)
        assert(res:get_body() == default_value)
        Response.del(res)
        Response.del(res)
    end)
end)
