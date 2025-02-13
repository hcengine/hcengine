---@type core
local core = require("engine.core")

---@class hc : core
local hc = require("hc.core")

local _print = print

local function watch(value, prefix, stack)
    if value == nil then
        return "<nil>"
    elseif type(value) == "function" then
        return "<function>"
    elseif type(value) == "boolean" then
        return (value and "true" or "false")
    elseif type(value) == "number" then
        return tostring(value)
    elseif type(value) == "userdata" then
        return tostring(value)
    elseif type(value) == "table" then
        local result = ""
        local sign = true
        local is_outer = false
    
        prefix = prefix or ""
        stack = stack or 0
        if not prefix then
            prefix = ""
        end

        if string.len(prefix) > 20 then
            result = "<table overflow>"
        else
            local str_list = { string.format("<table> {\r\n") }

            local size = 0
            for i, v in pairs(value) do
                size = size + 1
                if size > 100 then
                    if not is_outer then
                        table.insert(str_list, "... ...")
                        is_outer = true
                    end
                else
                    sign = true
                    if (type(i) == "string") and (type(v) == "table") then
                        -- 如果key值是以下划线开头，隐藏table的内容
                        -- 这个处理为避免上下级互相引用时出现死循环
                        local key = i

                        if (string.len(key) > 0) and (string.sub(key, 1, 1) == '_') then
                            table.insert(str_list, string.format("%s\t%s: <table hide>,\r\n",
                                        prefix, watch(i, prefix .. "\t", stack + 1)))
                            sign = false
                        end
                    end

                    if sign then
                        table.insert(str_list, string.format("%s\t%s:%s,\r\n", prefix,
                        watch(i, prefix .. "\t", stack + 1), watch(v, prefix .. "\t", stack + 1)))
                    end
                end

            end
            table.insert(str_list, string.format("%s} size: %d", prefix, size))
            result = table.concat(str_list, "")
        end
        return "\"" .. result .. "\""
    elseif type(value) == "string" then
        return "\"" .. value .. "\""
    end
    return "unknow"
end

local function trace(value, ...)
    assert(type(value) == "string", "必须为string做为格式化")
    local arg = {...}
    local i = 0
    local ret = string.gsub(value,"%%([o,s,d])",function(c)
        i = i+1
        if c == "s" then
            return arg[i]
        else
            return (watch(arg[i]))
        end
    end)

    print(ret)
end

hc.print = trace