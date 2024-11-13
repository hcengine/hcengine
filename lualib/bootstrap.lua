local hc = require("hc")
local hc1 = require("hc")
local hc2 = require("hc")

local new = require("hc.core");

print("bootstrap.lua ok!!!!!!!!")
print("name = %s", new.name)
print("id = %d", new.id)

hc.async(function()
    print("ok!!!!!!!!!!! this is co 11111111")
end)

print("2222222222")

hc.async(function()
    print("neeeeeeeeeeeeeeeeeee");
    --- @type ServiceConf
    local conf = ServiceConf.new();
    conf:set_from_table({
        creator = hc.id,
        name = "test",
        source = "test",
    });
    hc.new_service(conf)
end)
-- hc.close(0x01000001)
-- hc.exit(0)