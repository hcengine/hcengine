error("辅助编译器解读")

---@class core
---@field public id integer @service's id
---@field public name string @service's name
---@field public unique string @service's unique
local core = { }

--- let server exit: exitcode>=0 will wait all services quit.
---@param exitcode integer
function core.exit(exitcode) end

--- remove a service
--- @param service_id  integer 服务器id
function core.close(service_id) end


--- new service
--- @param conf ServiceConf 配置
function core.new_service(conf) end

--- 查询服务器id
--- @param name string 名字
--- @return integer
function core.query_service(name) end

return core