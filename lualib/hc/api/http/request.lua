error("辅助编译器解读")

--- lightuserdata, rust type `&mut Request`
---@class Request: object
Request = { }

---@param text string
function Request:set_text(text)
end

---@return string
function Request:header_get()
end

---@param key string
---@param val string
function Request:header_set(key, val)
end

---@param key string
function Request:header_remove(key)
end

function Request:header_clear()
end

---@return table
function Request:header_all()
end