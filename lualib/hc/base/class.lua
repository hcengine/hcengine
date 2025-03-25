-- class.lua
-- Lua实现多重继承的一种方案

local _class = {}

local unique_key = 0;

CLS_MAGIC_KEY = -1234567;

-- 取得新的 unique
local function new_unique()
    unique_key = unique_key + 1;
    return unique_key;
end

local all_cloned_obs = {}
setmetatable(all_cloned_obs, { __mode = "kv" })

-- 取得所有克隆的对象
function get_all_cloned_obs()
    return all_cloned_obs
end

-- 取得所有的对象按名称分组
function get_all_cloned_obs_group_by()
    local values = {}
    for _, ob in pairs(all_cloned_obs) do
        if type(ob) == "table" and ob.name and ob.is_ctor == true then
            values[ob.name] = values[ob.name] or {}
            table.insert(values[ob.name], ob)
        end
    end
    return values
end

-- 取得所有已析构的对象
function get_all_dtored_obs()
    local list = {}
    for _, ob in pairs(all_cloned_obs) do
        if type(ob) == "table" and ob.dtored == true then
            table.insert(list, ob)
        end
    end

    return list
end

function get_class()
    return _class
end

function clear_func_cache(c, func_name)
    if not _class[c._unique_key] then
        return
    end

    for k, _ in pairs(_class[c._unique_key]) do
        if func_name then
            if k == func_name then
                _class[c._unique_key][k] = nil
            end
        else
            if type(_class[c][k]) == "function" and k ~= "base" and k ~= "dtor_object" then
                _class[c._unique_key][k] = nil
            end
        end
    end
end

-- 取得指定类的指定函数
function get_class_func(c, func_name)
    local v = _class[c._unique_key]
    if not v then
        return;
    end

    return v[func_name]
end

function get_super(c, func_name)
    if #c.super > 0 then
        return get_class_func(c.super[1], func_name)
    end
    return nil
end

function call_super(c, func_name, ...)
    local func = get_super(c, func_name)
    if func then
        return func(...)
    end
end

function is_vaild(v)
    return v.is_ctor
end

-- 在 table plist 中查找 k
local function search(k, plist)
    for i = 1, #plist do
        -- 尝试第 i 个基类
        local v = _class[plist[i]._unique_key][k]

        -- 若基类中存在相关的值，则返回父类的值
        if v ~= CLS_MAGIC_KEY then
            return v
        end
    end
    return nil
end

---类定义的,名称为必须
---@param name string
---@param ... any
function class(name, ...)
    local class_type = {}
    class_type.name = name
    class_type._unique_key = new_unique()
    class_type.super = { ... }
    class_type.ob_list = {}
    class_type.ctor = false
    class_type.dtor = false
    setmetatable(class_type.ob_list, { __mode = "v" })

    -- 类对象创建函数
    class_type.new = function(...)
        local obj = { is_ctor = true }

        -- 这一句被我提前了，解决构造函数里不能调成员函数的问题
        -- 设置新对象的元表，其中的 index 元方法设置为一个父类方法查找表
        setmetatable(obj, { __index = _class[class_type._unique_key] })

        do
            local _ctor

            -- 创建对象时，依次调用父类的 ctor 函数
            _ctor = function(c, ...)
                if #c.super > 0 then
                    for i, v in ipairs(c.super) do
                        _ctor(v, ...)
                    end
                end
                if c.ctor then
                    c.ctor(obj, ...)
                end
            end

            _ctor(class_type, ...)
        end

        -- 记录创建的类对象
        class_type.ob_list[#class_type.ob_list + 1] = obj

        -- 将对象加入弱表中，用于内存泄漏的检测
        all_cloned_obs[#all_cloned_obs + 1] = obj
        return obj
    end

    -- 取得类对象接口函数
    class_type.get_func_list = function(c)
        local func_list = {}
        local _find

        _find = function(c, func_list)
            if #c.super > 0 then
                for i, v in pairs(c.super) do
                    _find(v, func_list)
                end
            end

            if _class[c._unique_key] then
                for k, v in pairs(_class[c._unique_key]) do
                    if v ~= CLS_MAGIC_KEY then
                        func_list[k] = v
                    end
                end
            end
        end

        _find(c, func_list)

        return func_list
    end

    -- 创建一个父类方法的查找表
    local vtbl = {
        class_type = name,
        get_class_func = get_class_func,
        get_class = get_class,
        get_super = get_super,
        call_super = call_super,
        is_vaild = is_vaild,
        CLS_MAGIC_KEY = CLS_MAGIC_KEY,
    }
    _class[class_type._unique_key] = vtbl

    -- 设置该类的 newindex 元方法
    setmetatable(class_type, {
        __newindex =
            function(t, k, v)
                vtbl[k] = v
            end
    })

    -- 类对象析构函数
    vtbl.dtor_object = function(obj)
        do
            local _dtor
            local dtor_table = {}
            -- 析构对象时，依次调用父类的 dtor 函数
            _dtor = function(c)
                if dtor_table[c.name] then
                    return
                end
                if c.dtor then
                    local status, e = pcall(c.dtor, obj)
                    if not status then
                        error(tostring(e))
                    end
                    dtor_table[c.name] = true
                end

                if #c.super > 0 then
                    for i = 1, #c.super do
                        _dtor(c.super[i])
                    end
                end
            end

            _dtor(class_type)
            obj.is_ctor = false
        end
    end

    -- 调用基类函数
    vtbl.base = function(obj, c, f, ...)
        -- 取得基类名+函数名的 key
        local k = string.format("%s%s", c.name, f)
        local ret = vtbl[k]

        if ret ~= CLS_MAGIC_KEY then
            -- 已存在该基类函数，直接调用
            local a, b, c, d, e = ret(obj, ...)
            return a, b, c, d, e
        end

        -- 遍历基类，查找函数
        if #c.super > 0 then
            for i = #c.super, 1, -1 do
                ret = search(f, c.super)
                if ret then
                    -- 取得基类函数，则调用之
                    vtbl[k] = ret
                    local a, b, c, d, e = ret(obj, ...)
                    return a, b, c, d, e
                end
            end
        end
    end

    -- 若该类有继承父类，则为父类查找表 vtbl 设置 index 元方法（查找父类的可用方法）
    if #class_type.super > 0 then
        setmetatable(vtbl, {
            __index =
                function(t, k)
                    local ret = search(k, class_type.super)
                    if not ret then
                        ---@diagnostic disable-next-line: assign-type-mismatch
                        vtbl[k] = CLS_MAGIC_KEY
                        return nil
                    elseif ret == CLS_MAGIC_KEY then
                        return nil
                    end

                    vtbl[k] = ret
                    return ret
                end
        })
    end

    return class_type
end

-- class_type = name,
-- get_class_func = get_class_func,
-- get_class = get_class,
-- get_super = get_super,
-- is_vaild = is_vaild,

---@class base_class
---@field class_type string
local base_class = {}
--- 实例化对象
--- @return any
base_class.new = function(...)

end

--- 获取所有的类
function base_class:get_class()
end

--- 获取所有函数
function base_class:get_class_func()
end

--- 获取父类函数
--- @param name string
function base_class:get_super(name)
end

--- 调用父类函数
--- @param name string
function base_class:call_super(name, ...)
end

--- 是否为合法
function base_class:is_vaild()
end

--- 构造函数
function base_class:ctor(...)
end

--- 析构函数
function base_class:dtor(...)
end

return class

--[[

现在，我们来看看怎么使用：

local base_type=class("base")   -- 定义一个基类 base_type

function base_type:ctor(x)      -- 定义 base_type 的构造函数
        print("base_type ctor")
        self.x=x
end

function base_type:print_x()    -- 定义一个成员函数 base_type:print_x
        print(self.x)
end

function base_type:hello()      -- 定义另一个成员函数 base_type:hello
        print("hello base_type")
end

以上是基本的 class 定义的语法，完全兼容 lua 的编程习惯。我增加了一个叫做 ctor 的词，作为构造函数的名字。
下面看看怎样继承：

local test=class("test", base_type)   -- 定义一个类 test 继承于 base_type

function test:ctor()    -- 定义 test 的构造函数
    print("test ctor")
end

function test:hello()   -- 重载 base_type:hello 为 test:hello
    print("hello test")
end

现在可以试一下了：

a=test.new(1)   -- 输出两行，base_type ctor 和 test ctor 。这个对象被正确的构造了。
a:print_x()     -- 输出 1 ，这个是基类 base_type 中的成员函数。
a:hello()       -- 输出 hello test ，这个函数被重载了。

--]]
