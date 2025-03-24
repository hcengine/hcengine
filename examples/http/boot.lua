local hc = require("lualib.hc")


hc.async(function()
    ---@type Router
    local router = hc.Router.new(function(req, res)
        res:set_status_code(201)
        local a = string.format("from lua!!!!!! %s", req:url())
        res:set_body(a)
        res:header_set("ok", "val")
        return res
    end)

    router:on("/test", function(req, res) 
        res:set_status_code(200)
        local a = string.format("test from lua!!!!!! %s", req:url())
        res:set_body(a)
        res:header_set("ok", "val")
        return res
    end)
    
    --@param req Request
    hc.bind_http("0.0.0.0:8082", router)
    -- --@type Request
    -- hc.timeout(1000, false, function()
    --     local req = Request.new()
    --     req:set_url("http://127.0.0.1:8082/startfromlua");

    --     local res, err = hc.http_request(req, nil)
    --     hc.print("receiver http msg")
    --     hc.print("res = %o, err = %o text = %o", res, err, res:get_text())
    --     Response.del(res)
    --     Response.del(res)
    --     -- hc.http_request(req, nil, function(res, err)
    --     --     hc.print("receiver http msg")
    --     --     hc.print("res = %o, err = %o text = %o", res, err, res:get_text())
    --     -- end)
    -- end)
end)