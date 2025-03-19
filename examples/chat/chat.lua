local hc = require("lualib.hc")

hc.print("this is example chat")

local online_users = {}

local service_id = nil

-- 广播消息, 将消息广播给所有人
local function broast_msg(id, msg)
    local text = hc.trim(msg:get_string() or "")
    if text == "close_service" then
        hc.close_socket(service_id or 0, "关闭连接");
        return
    end
    local from = online_users[id] or "unknown"
    local msg = NetMsg.pack_text(string.format("%s:%s", from, text))
    -- hc.print("开始广播数据:%o:%o:%o", id, text,online_users)
    for k, _ in pairs(online_users) do
        hc.send_msg(k, msg:clone_msg())
    end
end

hc.async(function()
    hc.listen("ws", "0.0.0.0:8090", {
        max_connections = 1025,
        read_timeout = 600000,
    }, {
        -- 子连接的接受参数, 并返回相应的回调数据, 若无则触发默认的回调函数
        on_accept = function(new_id, socket_addr)
            online_users[new_id] = socket_addr
            return {
                -- 收到子连接的消息
                ---@param msg NetMsg
                on_msg = function(id, msg)
                    hc.print("收到%o的消息, 类型为:%o", id, hc.get_net_name(msg:get_type()))
                    broast_msg(id, msg)
                end,
                -- 收到子链接关系的消息
                on_close = function(id, reason)
                    online_users[id] = nil
                    hc.print("收到关闭:%o 原因:%o", id, reason)
                end,
            }
        end,
        -- listen成功的消息
        on_open = function(id)
            hc.print("服务器绑定成功:%o", id)
            service_id = id
        end
    })
end)