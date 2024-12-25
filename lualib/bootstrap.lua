local hc = require("lualib.hc")
local hc1 = require("lualib.hc")
local hc2 = require("lualib.hc")

-- local xx = {a=21, b=false, c = {
--     d = "xxx",
--     f = function() end
-- }}
-- hc.print("%o", xx)

-- -- local new = require("engine.core");

-- -- print("bootstrap.lua ok!!!!!!!!")
-- -- print("name = %s", new.name)
-- -- print("id = %d", new.id)

-- -- hc.async(function()
-- --     print("ok!!!!!!!!!!! this is co 11111111")
-- -- end)

-- -- print("2222222222")

-- -- hc.async(function()
-- --     print("neeeeeeeeeeeeeeeeeee", hc.id);
-- --     --- @type ServiceConf
-- --     local conf = ServiceConf.new();
-- --     conf:set_from_table({
-- --         name = "test",
-- --         source = "test",
-- --     });
-- --     local id, err = hc.new_service(conf)
-- --     print("id === ", id, err)
-- --     local id2, err = hc.new_service(conf)
-- --     print("id === ", id2, err)
-- --     local h3 = hc.query_service("test")
-- --     assert(id == h3, "id must equal")

-- --     local v, x = hc.call("lua", id, "sum", 6, 3)
-- --     print("val == ", v, x)

-- --     -- local h4 = hc.query_service("bootstrap")
-- --     -- assert(h4 == 1, "id must equal")
-- -- end)

-- -- -- for key, value in pairs(LuaMsg) do
-- -- --     print("LuaMsg ddddddddddd ", key, value)
-- -- -- end
-- -- -- for key, value in pairs(Protocol) do
-- -- --     print("Protocol ddddddddddd ", key, value)
-- -- -- end
-- -- -- print("aaaaaaaaaaaa", Protocol)
-- -- -- print("aaaaaaaaaaaa", Protocol.lua_pack)
-- -- -- local msg = hc.pack(1, 2, "aa", {ff = "11"})
-- -- -- local a, b, c, d = hc.unpack(msg);
-- -- -- print("ccccccccc", a, b, c, d)

-- -- -- hc.close(0x01000001)
-- -- -- hc.exit(0)

-- -- local id = hc.timeout(10, false, function ()
-- --     error("timeout aaaa")
-- -- end)

-- -- hc.del_timer(id)

-- -- hc.async(function()
-- --     print("ccaaaa");
-- --     hc.sleep(20)
-- --     print("xxxxx");
-- -- end)

-- hc.print("value = %o", hc.args())
-- -- hc.print("valu = %o", hc.env("PATH"))
-- hc.print("value = %o", hc.env("zz"))

hc.async(function()
    -- hc.print("cxxxxxxxxxxxxxxxx ret = %d", 0)
    local ret = hc.wait(hc.listen("ws", "0.0.0.0:2003", {
        max_connections = 1025,
        cert = "key/xx.pem",
    }, {
        on_accept = function(new_id)
            hc.print("id === %o", new_id)
            return {
                on_msg = function(id, msg)
                    hc.print("on_accept ret callback on_msg === %o, msg = %o", id, msg:get_string())
                    hc.send_msg(id, NetMsg.pack_text(msg:get_string() or "empty"))
                end,
                on_open = function(id)
                    hc.print("on_open ret on close!!!!!!!!!! %o", id)
                end,
                on_close = function(id)
                    hc.print("on_accept ret on close!!!!!!!!!! %o", id)
                end,
            }
        end,
        on_msg = function(id, msg)
            hc.print("on_msg === %o, msg = %o", id, msg:get_string())
            hc.send_msg(id, msg)
            return true
        end,
    }))
    hc.print("cxxxxxxxxxxxxxxxx ret = %d", ret)
    
    local send = NetMsg.pack_text(string.format("from lua %s", "a"))
    local meta = getmetatable(send)
    hc.print("send = %o meta = %o", send, meta)
    hc.send_msg(0, send)
    -- hc.send_msg1(send, 0)
    hc.print("xxxxxxxxxxxxxx")
    -- hc.send_msg(0, send)
end)