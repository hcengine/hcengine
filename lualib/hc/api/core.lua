error("辅助编译器解读")

--- lightuserdata, rust type `&mut LuaMsg`
---@class LuaMsg
---@field public ty integer
---@field public sender integer
---@field public receiver integer
---@field public sessionid integer
---@field public get_ty fun(self: LuaMsg): integer
LuaMsg = {
    del = function(msg)
    end
}