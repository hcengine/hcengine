error("辅助编译器解读")

--- lightuserdata, rust type `&mut LuaMsg`
---@class LuaMsg: object
---@field public ty integer
---@field public sender integer
---@field public receiver integer
---@field public sessionid integer
---@field public get_ty fun(self: LuaMsg): integer
LuaMsg = { }

---@return integer
function LuaMsg:read_u64()
end

---@return integer
function LuaMsg:read_i64()
end

---@return number
function LuaMsg:read_f32()
end

---@return number
function LuaMsg:read_f64()
end

---@return string
function LuaMsg:read_str()
end

---@return string
function LuaMsg:get_err()
end

