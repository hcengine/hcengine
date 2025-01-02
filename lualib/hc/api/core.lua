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

--- call
--- @param msg LuaMsg 消息
function core.send(msg) end

--- 返回消息
--- @param msg LuaMsg 消息
function core.resp(msg) end

--- 超时消息
--- @param interval integer
--- @param is_repeat boolean
--- @return integer
function core.timeout(interval, is_repeat) end

--- 超时消息
--- @param timer_id integer
function core.del_timer(timer_id) end

--- 当时时间秒
--- @return integer
function core.now() end

--- 当时时间毫秒
--- @return integer
function core.now_ms() end

--- 当前环境变量
--- @param arg string
--- @return string | string[]
function core.env(arg) end


--- 绑定连接
--- @param method string
--- @param url string
--- @param settings table | nil
--- @return integer
function core.bind_listen(method, url, settings) end

--- 发起连接
--- @param method string
--- @param url string
--- @param settings table | nil
--- @return integer
function core.connect(method, url, settings) end


--- 关闭连接
--- @param id integer
--- @param reason string
function core.close_socket(id, reason) end


--- 发送消息
--- @param id integer
--- @param msg NetMsg
function core.send_msg(id, msg) end

return core