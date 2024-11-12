error("辅助编译器解读")

---@class object
local obj = { }

---@return any
function obj.new() return obj end

---@param v object
function obj.del(v) end

---@param v table
function obj:set_from_table(v) end

return obj
