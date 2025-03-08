local core = require("engine.core")
local _set_mysql_url = core.set_mysql_url
local _run_mysql_only = core.run_mysql_only
local _run_mysql_one = core.run_mysql_one
local _run_mysql_query = core.run_mysql_query
local _run_mysql_iter = core.run_mysql_iter
local _run_mysql_insert = core.run_mysql_insert
local _run_mysql_update = core.run_mysql_update
local _run_mysql_ignore = core.run_mysql_ignore

---@class hc : core
local hc = require("hc.core")

local mysql = {}

mysql.index = 0

---设置默认mysql的索引
---@param index integer
function mysql:set_mysql_index(index)
    self.index = index
end

---设置默认mysql的url映射索引
---@param url string
function mysql:set_mysql_url(url)
    self.index = _set_mysql_url(url)
    return self.index
end

---运行mysql命令
function mysql:run_mysql_only(...)
    return mysql.run_mysql_only_with_index(self.index, ...)
end

---运行mysql根据索引
function mysql.run_mysql_only_with_index(index, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    return hc.wait(_run_mysql_only(index, ...))
end

---运行mysql命令
function mysql:run_mysql_one(...)
    return mysql.run_mysql_one_with_index(self.index, ...)
end

---运行mysql根据索引
function mysql.run_mysql_one_with_index(index, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    return hc.wait(_run_mysql_one(index, ...))
end


---运行mysql命令
function mysql:run_mysql_query(...)
    return mysql.run_mysql_query_with_index(self.index, ...)
end

---运行mysql根据索引
function mysql.run_mysql_query_with_index(index, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    return hc.wait(_run_mysql_query(index, ...))
end

---运行mysql命令
function mysql:run_mysql_iter(fn, ...)
    return mysql.run_mysql_iter_with_index(self.index, fn, ...)
end

---运行mysql根据索引
function mysql.run_mysql_iter_with_index(index, fn, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    local session = _run_mysql_iter(index, ...)
    local cols = nil
    local col_len = 0
    while true do
        local ret, err = hc.wait(session)
        if not ret or err then
            fn(ret, err or "not any result")
            break
        end
        if not cols then
            cols = ret
            col_len = #cols
        else
            local ret_table = {}
            for i = 1,col_len do
                ret_table[cols[i]] = ret[i]
            end
            fn(ret_table, err)
        end
    end
end

---运行mysql命令
function mysql:run_mysql_insert(...)
    return mysql.run_mysql_insert_with_index(self.index, ...)
end

---运行mysql根据索引
function mysql.run_mysql_insert_with_index(index, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    return hc.wait(_run_mysql_insert(index, ...))
end

---运行mysql命令
function mysql:run_mysql_update(...)
    return mysql.run_mysql_update_with_index(self.index, ...)
end

---运行mysql根据索引
function mysql.run_mysql_update_with_index(index, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    return hc.wait(_run_mysql_update(index, ...))
end

---运行mysql命令
function mysql:run_mysql_ignore(...)
    return mysql.run_mysql_ignore_with_index(self.index, ...)
end

---运行mysql根据索引
function mysql.run_mysql_ignore_with_index(index, ...)
    hc.print("run_mysql_command_with_index args = %o", {index, ...})
    return hc.wait(_run_mysql_ignore(index, ...))
end

return mysql