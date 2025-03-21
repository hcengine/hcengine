---@type core
local core = require("engine.core")
---@class hc : core
local hc = require("hc.core")

local cjson = require("cjson")

-- 删除前后空白符
hc.trim = function(s)
    return (string.gsub(s, "^%s*(.-)%s*$", "%1"))
end

-- 根据规则删除空白符
hc.trim_reg = function(s, reg)
    return (string.gsub(s, "^" .. reg .. "*(.-)" .. reg .. "*$", "%1"))
end


-- 将字符串根据标识符打断，组成 array
hc.explode = function(str, flag)
    local t, ll
    t = {}
    ll = 0

    if #str == 0 then
        return {}
    end

    if (#str == 1) then
        return { str }
    end

    local l
    while true do
        l = string.find(str, flag, ll, true)

        if l ~= nil then
            table.insert(t, string.sub(str, ll, l - 1))
            ll = l + 1
        else
            table.insert(t, string.sub(str, ll))
            break
        end
    end

    return t
end

-- 将字符串根据标识符打断，组成 数字array
hc.explode_tonumber = function(str, flag)
    local t = hc.explode(str, flag)
    local result = {}
    for _, v in ipairs(t) do
        result[#result + 1] = tonumber(v)
    end
    return result
end


-- 转化成JSON格式
hc.encode_json = function(t)
    if type(t) ~= "table" then
        return "{}"
    end
    local success, ret = pcall(cjson.encode, t)
    if success then
        return ret
    else
        return "{}"
    end
end


hc.decode_json = function(s, max_len)
    if type(s) ~= "string" then
        return {}
    end
    if max_len and #s > max_len then
        return {}
    end
    if #s == 0 then
        return {}
    end
    local success, ret = pcall(cjson.decode, s)
    if type(ret) ~= "table" then
        success = false
    end
    if success then
        return ret
    else
        return {}
    end
end

hc.create_query_string = function(form)
    local array = {}
    for k, v in pairs(form) do
        table.insert(array, string.format("%s=%s", hc.url_encode(k), hc.url_encode(tostring(v))))
    end
    return table.concat(array, "&")
end

hc.url_decode = function(str)
    str = string.gsub(str, "+", " ")
    str = string.gsub(str, "%%(%x%x)",
        function(h) return string.char(tonumber(h, 16)) end)
    str = string.gsub(str, "\r\n", "\n")
    return str
end

hc.url_encode = function(str)
    if (str) then
        str = string.gsub(str, "\n", "\r\n")
        str = string.gsub(str, "([^%w %-%_%.%~])",
            function(c) return string.format("%%%02X", string.byte(c)) end)
        str = string.gsub(str, " ", "+")
    end
    return str
end
