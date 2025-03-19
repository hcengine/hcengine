---@type core
local core = require("engine.core")
---@class hc : core
local hc = require("hc.core")

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
