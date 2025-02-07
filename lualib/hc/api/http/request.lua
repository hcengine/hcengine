error("辅助编译器解读")

--- lightuserdata, rust type `&mut Request`
---@class Request: object
Request = { }


---@param key string
---@param val string
function Request:header_set(key, val)
end

---@param text string
function Request:set_text(text)
end
