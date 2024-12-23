error("辅助编译器解读")

---@class NetMsg: object
NetMsg = { }

---@param self NetMsg
---@return integer
function NetMsg:get_type() end

---@return string
function NetMsg:get_string() end

---@return string
function NetMsg:get_lstring() end

---@return string
function NetMsg:take_data() end

---@param text string
---@return NetMsg
function NetMsg.pack_text(text) end

---@return NetMsg
function NetMsg.pack_binary(raw) end

---@return NetMsg
function NetMsg.pack_ping(raw) end

---@return NetMsg
function NetMsg.pack_pong(raw) end

return NetMsg