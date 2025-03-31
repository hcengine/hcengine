---@class hc : core
local hc = require("hc.core")

---@class Router: base_class
local Router = hc.class("Router")

local all_methods = {
    GET = true,
    POST = true,
    OPTION = true,
    DELETE = true,
    PUT = true,
}

local default_callback = function(req, res)
    res:set_status_code(400)
    res:set_body("未设置正确的处理函数")
end

function Router:ctor(func)
    hc.print("Router:ctor!!!!!!!!!!!!!")
    self.paths = {}
    self.matchs = {}
    self.default = func or default_callback
    -- hc.print("self.xx = %o", self.xx)
end

function Router:dtor()

end

function Router:combine_method_router(method, router)
    return string.format("%s:%s", method, router)
end

function Router:on_default(func)
    self.default = func
end

function Router:on(name, func, methods)
    local m = {}
    if type(methods) == "string" then
        m[methods] = true
    elseif type(methods) == "table" then
        for _, k in ipairs(methods) do
            m[k] = true
        end
    elseif methods == nil then
        m = all_methods
    end
    for k, _ in pairs(m) do
        local r = self:combine_method_router(k, name)
        self.paths[r] = func
    end
end

function Router:on_reg(name, func, methods)
    hc.print("self === %o", self)
    local m = {}
    if type(methods) == "string" then
        m[methods] = true
    elseif type(methods) == "table" then
        for _, k in ipairs(methods) do
            m[k] = true
        end
    elseif methods == nil then
        m = all_methods
    end
    self.matchs[name] = {
        func = func,
        methods = m
    }
end

---@param req Request
---@return Response
function Router:call(req)
    local res = Response:new()
    hc.print("req = %o", req)
    hc.print("path = %o", req:path())
    local path = req:path();
    local method = req:method()
    hc.print("self.default = %o method = %o", self.default, method)
    local mr = self:combine_method_router(method, path)
    local f = self.paths[mr]
    if f then
        f(req, res)
        return res
    else
        for k, t in pairs(self.matchs) do
            if string.find(path, k) then
                hc.print("k = %o, path = %o, res = %o methods = %o", k, path, string.find(path, k), t["methods"])
                if t["methods"][method] then
                    t["func"](req, res)
                    return res
                end
            end
        end
        self.default(req, res)
    end
    return res
end

return Router
