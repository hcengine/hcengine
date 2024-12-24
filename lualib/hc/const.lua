local core = require("engine.core")
---@class hc : core
local hc = require("hc.core")


hc.NET_TEXT = 1
hc.NET_BINARY = 2
hc.NET_CLOSE = 8
hc.NET_PING = 9
hc.NET_PONG = 10
hc.NET_SHUTDOWN = 11
hc.NET_BAD = 255

hc.NET_NAMES = {
    [hc.NET_TEXT] = "text",
    [hc.NET_BINARY] = "binary",
    [hc.NET_CLOSE] = "close",
    [hc.NET_PING] = "ping",
    [hc.NET_PONG] = "pong",
    [hc.NET_SHUTDOWN] = "shutdown",
    [hc.NET_BAD] = "bad",
}