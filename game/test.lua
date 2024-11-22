local hc = require("lualib.hc")


function sum(s, e)
    print("sum call ~~~~~~~~~~", s, e)
    return "s - e", s + e
end

print("test!!!!", hc.id)