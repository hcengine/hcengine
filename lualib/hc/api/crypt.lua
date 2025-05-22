error("辅助编译器解读")

---@class crypt
local crypt = {}

---@param val string
---@return string
function crypt.md5(val) end

---@param key string
---@param val string
---@return string
function crypt.hmac_md5(key, val) end

---@param key string
---@param val string
---@return string
function crypt.hmac_sha256(key, val) end

---@param val string
---@return string
function crypt.base64_encode(val) end

---@param val string
---@return string | nil
function crypt.base64_decode(val) end

return crypt

