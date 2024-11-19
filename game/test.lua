local hc = require("hc")


function sum(s, e)
    print("sum call ~~~~~~~~~~", s, e)
    return s - e
end

print("test!!!!", hc.id)