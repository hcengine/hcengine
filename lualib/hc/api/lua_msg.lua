error("辅助编译器解读")

--- lightuserdata, rust type `&mut LuaMsg`
---@class LuaMsg: object
---@field public ty integer
---@field public sender integer
---@field public receiver integer
---@field public sessionid integer
---@field public get_ty fun(self: LuaMsg): integer
LuaMsg = { }

---@return boolean
function LuaMsg:read_bool()
end

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

---@param val boolean
function LuaMsg:write_bool(val)
end

---@param val integer
function LuaMsg:write_u64(val)
end

---@param val integer
function LuaMsg:write_i64(val)
end

---@param val number
function LuaMsg:write_f32(val)
end

---@param val number
function LuaMsg:write_f64(val)
end

---@param val string
function LuaMsg:write_str(val)
end

---@return string
function LuaMsg:get_err()
end

