error("辅助编译器解读")

--- lightuserdata, rust type `&mut Request`
---@class Request: object
Request = { }

---@return string
function Request:method()
end

---@param method string | "'GET'" | "'POST'" | "'OPTION'" | "'DELETE'" | "'PUT'"
function Request:set_method(method)
end

---@param text string
function Request:write(text)
end

---@param text string
function Request:set_body(text)
end

---@return string
function Request:url()
end

---@param url string
function Request:set_url(url)
end

---@return string
function Request:query()
end

---@param query string
function Request:set_query(query)
end

---@return string
function Request:port()
end

---@param port integer
function Request:set_port(port)
end

---@return string
function Request:path()
end

---@param path string
function Request:set_path(path)
end

---@return boolean
function Request:is_http2()
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



