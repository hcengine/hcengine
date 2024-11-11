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

hc.close(0)
hc.exit(0)