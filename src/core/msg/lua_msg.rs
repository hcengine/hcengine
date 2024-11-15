use algorithm::buf::{BinaryMut, Bt};
use hclua::{lua_State, Lua, LuaObject, ObjectMacro};

#[derive(Default, ObjectMacro)]
#[hclua_cfg(name = LuaMsg)]
#[hclua_cfg(light)]
pub struct LuaMsg {
    pub ty: u8,
    pub sender: u32,
    pub receiver: u32,
    pub sessionid: i64,
    pub err: Option<String>,
    #[hclua_skip]
    pub data: BinaryMut,
}

impl LuaMsg {
    pub fn new(ty: u8, data: BinaryMut) -> Self {
        LuaMsg {
            ty,
            sender: 0,
            receiver: 0,
            sessionid: 0,
            err: None,
            data,
        }
    }

    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        LuaObject::<LuaMsg>::object_def(lua, "read_u64", hclua::function1(Self::read_u64));
        LuaObject::<LuaMsg>::object_def(lua, "read_i64", hclua::function1(Self::read_i64));
        LuaObject::<LuaMsg>::object_def(lua, "read_f32", hclua::function1(Self::read_f32));
        LuaObject::<LuaMsg>::object_def(lua, "read_f64", hclua::function1(Self::read_f64));
        LuaObject::<LuaMsg>::object_def(lua, "read_str", hclua::function1(Self::read_str));
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        self.data.try_get_u64().ok()
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        self.data.try_get_i64().ok()
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        self.data.try_get_f32().ok()
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        self.data.try_get_f64().ok()
    }

    pub fn read_str(&mut self) -> Option<String> {
        hcproto::decode_string(&mut self.data).ok()
    }
}
