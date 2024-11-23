local hc = require("lualib.hc")
local hc1 = require("lualib.hc")
local hc2 = require("lualib.hc")


-- local new = require("engine.core");

-- print("bootstrap.lua ok!!!!!!!!")
-- print("name = %s", new.name)
-- print("id = %d", new.id)

-- hc.async(function()
--     print("ok!!!!!!!!!!! this is co 11111111")
-- end)

-- print("2222222222")

-- hc.async(function()
--     print("neeeeeeeeeeeeeeeeeee", hc.id);
--     --- @type ServiceConf
--     local conf = ServiceConf.new();
--     conf:set_from_table({
--         name = "test",
--         source = "test",
--     });
--     local id, err = hc.new_service(conf)
--     print("id === ", id, err)
--     local id2, err = hc.new_service(conf)
--     print("id === ", id2, err)
--     local h3 = hc.query_service("test")
--     assert(id == h3, "id must equal")

--     local v, x = hc.call("lua", id, "sum", 6, 3)
--     print("val == ", v, x)

--     -- local h4 = hc.query_service("bootstrap")
--     -- assert(h4 == 1, "id must equal")
-- end)

-- -- for key, value in pairs(LuaMsg) do
-- --     print("LuaMsg ddddddddddd ", key, value)
-- -- end
-- -- for key, value in pairs(Protocol) do
-- --     print("Protocol ddddddddddd ", key, value)
-- -- end
-- -- print("aaaaaaaaaaaa", Protocol)
-- -- print("aaaaaaaaaaaa", Protocol.lua_pack)
-- -- local msg = hc.pack(1, 2, "aa", {ff = "11"})
-- -- local a, b, c, d = hc.unpack(msg);
-- -- print("ccccccccc", a, b, c, d)

-- -- hc.close(0x01000001)
-- -- hc.exit(0)

-- local id = hc.timeout(10, false, function ()
--     error("timeout aaaa")
-- end)

-- hc.del_timer(id)

-- hc.async(function()
--     print("ccaaaa");
--     hc.sleep(20)
--     print("xxxxx");
-- end)