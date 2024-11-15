error("辅助编译器解读")

--- lightuserdata, rust type `&mut LuaMsg`
---@class Protocol: object
Protocol = { }

--- 任意参数序列化
--- @return LuaMsg
Protocol.lua_pack = function(...) end

--- 任意参数反序列化
--- @param msg LuaMsg
Protocol.lua_unpack = function(msg) end