error("辅助编译器解读")

---@class NetMsg: object
NetMsg = { }

---@param self NetMsg
---@return integer
function NetMsg:get_type() end

---@return string
function NetMsg:get_string() end

---@param text string
---@return NetMsg
function NetMsg.pack_text(text) end


return NetMsg