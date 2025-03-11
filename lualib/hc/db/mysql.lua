local core = require("engine.core")
local _set_mysql_url = core.set_mysql_url
local _run_mysql_only = core.run_mysql_only
local _run_mysql_one = core.run_mysql_one
local _run_mysql_query = core.run_mysql_query
local _run_mysql_iter = core.run_mysql_iter
local _run_mysql_insert = core.run_mysql_insert
local _run_mysql_update = core.run_mysql_update
local _run_mysql_ignore = core.run_mysql_ignore
local _get_mysql_keep = core.get_mysql_keep
local _del_mysql_keep = core.del_mysql_keep

---@class hc : core
local hc = require("hc.core")

---@class mysql
local mysql = {}

mysql.index = 0
mysql.keep = 0

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
    hc.print("index = %o", self.index)
    return hc.wait(_run_mysql_only(self.index, self.keep, ...))
end

---运行mysql命令
function mysql:run_mysql_one(...)
    return hc.wait(_run_mysql_one(self.index, self.keep, ...))
end

---运行mysql命令
function mysql:run_mysql_query(...)
    return hc.wait(_run_mysql_query(self.index, self.keep, ...))
end

---运行mysql命令
---@param fn fun(val: table|nil, err:nil)
function mysql:run_mysql_iter(fn, ...)
    hc.print("run_mysql_command_with_index args = %o", { self.index, self.keep, ... })
    local session = _run_mysql_iter(self.index, self.keep, ...)
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
            for i = 1, col_len do
                ret_table[cols[i]] = ret[i]
            end
            fn(ret_table, err)
        end
    end
end

---运行mysql命令
function mysql:run_mysql_insert(...)
    return hc.wait(_run_mysql_insert(self.index, self.keep, ...))
end

---运行mysql命令
function mysql:run_mysql_update(...)
    return hc.wait(_run_mysql_update(self.index, self.keep, ...))
end

---运行mysql命令
function mysql:run_mysql_ignore(...)
    return hc.wait(_run_mysql_ignore(self.index, self.keep, ...))
end

---运行mysql命令
function mysql:get_mysql_keep()
    return hc.wait(_get_mysql_keep(self.index))
end

---运行mysql命令
function mysql:del_mysql_keep()
    return _del_mysql_keep(self.index, self.keep)
end

---@return mysql
function mysql:build_connect(table)
    setmetatable(table, { __index = self })
    return table
end

return mysql
