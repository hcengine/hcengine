error("辅助编译器解读")

--- lightuserdata, rust type `&mut Response`
---@class Response: object
Response = { }


---@param code integer
function Response:set_status_code(code)
end

---@return integer
function Response:status_code()
end

---@return string
function Response:status_str()
end


---@return string
function Response:version()
end

---@param text string
function Response:write(text)
end

---@param text string
function Response:set_body(text)
end

function Response:get_body()
end

---@return string
function Response:header_get()
end

---@param key string
---@param val string
function Response:header_set(key, val)
end

---@param key string
function Response:header_remove(key)
end

function Response:header_clear()
end

---@return table
function Response:header_all()
end