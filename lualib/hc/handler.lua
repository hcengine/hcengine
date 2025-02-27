---@class hc : core
local hc = require("hc.core")

-- 自定义调用栈输出
function hc.traceback(log_to_file)
    local result = { "stack traceback:\n", }

    local i = 3
    local j = 1
    local file, line, func, var, value

    -- 遍历所有调用层次
    local source_info
    local debug_get_info = debug.getinfo
    local debug_get_local = debug.getlocal
    repeat
        source_info = debug_get_info(i,'Sln') -- .source
        if not source_info then
            do break end
        end

        -- 取得文件名、行号、函数名信息
        file = source_info.short_src or ""
        line = source_info.currentline or ""
        func = source_info.name or ""

        table.insert(result, string.format("\t(%d)%s:%s: in function '%s'\n",
                                           i - 2, file, line, func))
        if source_info.what ~= "C" and
           func ~= "_create" and func ~= "_destruct" and func ~= "new" then
            -- 遍历该层次的所有 local 变量
            j = 1
            repeat
                var, value = debug_get_local(i, j)
                if var and not string.find(var, "%b()") then
                    if value then
                        table.insert(result, string.format("\t\t%s : %s\n", tostring(var),
                                                           hc.watch(value, "\t\t", 1)))
                    else
                        table.insert(result, string.format("\t\t%s : <nil>\n", tostring(var)))
                    end
                end

                j = j + 1
            until not var
        end

        i = i + 1
    until not source_info

    local str = table.concat(result, "")
    return str
end

-- 重新定义assert函数，打印调用栈
function hc.assert(e, msg)
    if not e then
        hc.traceback(true)

        local err = string.format("Assert Failed: %s\n", tostring(msg))
        error(err)

    end
end

-- 异常处理函数，打印调用栈
function hc.error_handle(...)
    local err_msg = ...
    if type(err_msg) == "table" then
        err_msg = err_msg[1]
    end

    hc.traceback(true)
    err_msg = string.format( "Error:\n%s\n", err_msg)
    hc.print( "%s", err_msg )
    return ""
end


__G__TRACKBACK__ = hc.error_handle
_G["ERROR_HANDLE"] = hc.error_handle