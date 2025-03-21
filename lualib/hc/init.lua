---@class hc : core
local hc = require("hc.core")
require("hc.util")

hc.class = require("hc.base.class")

require("hc.const")
require("hc.timer")
require("hc.trace")
require("hc.net")
require("hc.http.http")
require("hc.handler")

require("hc.db.redis")


hc.init()
return hc