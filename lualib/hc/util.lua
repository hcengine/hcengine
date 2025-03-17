---@type core
local core = require("engine.core")
---@class hc : core
local hc = require("hc.core")

hc.trim = function(s)
    return (string.gsub(s, "^%s*(.-)%s*$", "%1"))
end

hc.trim_reg = function(s, reg)
    return (string.gsub(s, "^" .. reg .. "*(.-)" .. reg .. "*$", "%1"))
end