error("辅助编译器解读")

---@class net_cb
---@field public on_msg fun(id: integer, msg: NetMsg):(boolean|nil) | nil
---@field public on_accept fun(id: integer, socket_addr: string):(net_cb|nil) | nil 
---@field public on_close fun(id: integer, reason: string) | nil
---@field public on_open fun(id: integer) | nil
---@field public on_ping fun(id: integer, data: string) | nil
---@field public on_pong fun(id: integer, data: string) | nil
local net_cb = { }

---@class net_settings
---@field max_connections integer | nil 最大的连接上限 1024
---@field queue_size integer | nil 默认队列大小 10
---@field in_buffer_max integer | nil 读数据的最大容量 10M
---@field out_buffer_max integer | nil 写数据的最大容量 10M
---@field onemsg_max_size integer | nil 单信息最大的数量 65535
---@field closing_time integer | nil 最关闭状态下留给写入的最长时间 1000ms
---@field connect_timeout integer | nil 连接的最大时长 30000ms
---@field shake_timeout integer | nil 连接的最大时长 30000ms
---@field read_timeout integer | nil 连接的最大时长 30000ms
---@field is_raw boolean | nil 是否为raw传输，即tcp默认不分包 false
---@field domain string | nil TLS证书所用域名, 如果有该变量则表示开启
---@field cert string | nil 证书的公钥文件
---@field key string | nil 证书的私钥文件
local net_settings = {}

return net_cb