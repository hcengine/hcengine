error("辅助编译器解读")

--- lightuserdata, rust type `&mut Response`
---@class Response: object
Response = { }

---@param key string
---@param val string
function Response:header_set(key, val)
end

---@param text string
function Response:set_text(text)
end
