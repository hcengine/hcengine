local hc = require("lualib.hc")

local default_value = "default return"
local test_value_get = "test return get"
local test_value_post = "test return post"
local test_reg = "test reg"

-- local ret, err = pcall(function() 
--     hc.print("set_loglevelset_loglevelset_loglevel")
--     hc.print("func === %oaa = %o", type(error_handle), hc.set_loglevel("aaaa"))
--     hc.print("aaaaaaaaaaaaaaaa")
-- end)
hc.async(function() 
    local ret, err = xpcall(function()
        hc.print("aaaaa 1")
        hc.env("path")
        hc.print("bbbb 2")
    end, ERROR_HANDLE)
    hc.print("outer")
end)

-- hc.set_loglevel(nil)

-- local ret, err = xpcall(function()
--     hc.print("aaaaa")
--     hc.bind_http("0.0.0.0:8082")
--     hc.print("bbbb")
-- end, ERROR_HANDLE)





-- error("aaaaaaaaaaaaaa")

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
        res:set_body(test_value_get)
        res:header_set("ok", "val")
        return res
    end, "GET")

    router:on("/test", function(req, res)
        res:set_status_code(200)
        res:set_body(test_value_post)
        res:header_set("ok", "val")
        return res
    end, "POST")

    router:on_reg("/test(%w+)", function(req, res)
        res:set_status_code(200)
        res:set_body(test_reg)
        res:header_set("ok", "val")
        return res
    end)
    -- error("aaaaaaaaaaaaaa")
    --@param req Request
    hc.bind_http("0.0.0.0:8082", router, 60)
    hc.timeout(1000, false, function()
        local res, err = hc.http_get("http://127.0.0.1:8082/")
        assert(res ~= nil and err == nil)
        assert(res:get_body() == default_value)
        Response.del(res)
        hc.print("check / success")
    end)
    hc.timeout(1000, false, function()
        local res, err = hc.http_get("http://127.0.0.1:8082/test")
        assert(res ~= nil and err == nil)
        assert(res:get_body() == test_value_get)
        Response.del(res)
        hc.print("check /test success")
    end)
    
    hc.timeout(1000, false, function()
        local res, err = hc.http_post("http://127.0.0.1:8082/test", "")
        assert(res ~= nil and err == nil)
        assert(res:get_body() == test_value_post)
        Response.del(res)
        hc.print("check /test post success")
    end)
    
    hc.timeout(1000, false, function()
        --- @type Request
        local req = Request.new()
        req:set_url("http://127.0.0.1:8082/testaaaaaa");
        local res, err = hc.http_request(req, nil)
        assert(res ~= nil and err == nil)
        assert(res:get_body() == test_reg)
        Response.del(res)
        hc.print("check /testaaaaaa success")
    end)
end)

