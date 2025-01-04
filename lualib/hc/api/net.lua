error("辅助编译器解读")

---@class net_cb
---@field public on_msg fun(id: integer, msg: NetMsg):(boolean|nil) | nil
---@field public on_accept fun(id: integer):(net_cb|nil) | nil 
---@field public on_close fun(id: integer) | nil
---@field public on_open fun(id: integer) | nil
---@field public on_ping fun(id: integer, data: string) | nil
---@field public on_pong fun(id: integer, data: string) | nil
local net_cb = { }


return net_cb