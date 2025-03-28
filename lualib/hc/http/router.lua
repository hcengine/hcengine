---@class hc : core
local hc = require("hc.core")

---@class Router: base_class
local Router = hc.class("Router")

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

function Router:on_default(func)
    self.default = func
end

function Router:on(name, func)
    hc.print("self === %o", self)
    self.paths[name] = func
end

function Router:on_reg(name, func)
    hc.print("self === %o", self)
    self.matchs[name] = func
end

---@param req Request
---@return Response
function Router:call(req)
    local res = Response:new()
    hc.print("req = %o", req)
    hc.print("path = %o", req:path())
    hc.print("self.default = %o", self.default)
    local path = req:path();
    local func = self.paths[path]
    if func then
        func(req, res)
    else
        for k, v in pairs(self.matchs) do
            hc.print("k = %o, path = %o, res = %o", k, path, string.find(path, k))
            if string.find(path, k) then
                v(req, res)
                return res
            end
        end
        self.default(req, res)
    end
    return res
end

return Router
