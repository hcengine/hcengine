use std::fmt::Write;

use algorithm::buf::{BinaryMut, Bt, BtMut};
use hclua::{lua_State, Lua, LuaObject, LuaPush, ObjectMacro};

use crate::{core::wrapper::WrapperLuaMsg, Config};

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
    #[hclua_skip]
    // pub obj: Option<Box<()>>,
    pub obj: Option<WrapperLuaMsg>,
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
            obj: None,
        }
    }

    pub fn new_integer(val: i64, service_id: u32, session: i64) -> Self {
        let mut data = BinaryMut::new();
        data.put_i64(val);
        LuaMsg {
            ty: Config::TY_INTEGER,
            receiver: service_id,
            sessionid: session,
            data,
            ..Default::default()
        }
    }

    pub fn new_number(val: f64, service_id: u32, session: i64) -> Self {
        let mut data = BinaryMut::new();
        data.put_f64(val);
        LuaMsg {
            ty: Config::TY_NUMBER,
            receiver: service_id,
            sessionid: session,
            data,
            ..Default::default()
        }
    }

    pub fn new_string(val: &str, service_id: u32, session: i64) -> Self {
        let mut data = BinaryMut::new();
        data.write_str(val).expect("ok");
        LuaMsg {
            ty: Config::TY_STRING,
            receiver: service_id,
            sessionid: session,
            data,
            ..Default::default()
        }
    }

    pub fn new_timer(timer_id: u64, is_repeat: bool, service_id: u32) -> Self {
        let mut data = BinaryMut::new();
        data.put_u64(timer_id);
        data.put_bool(is_repeat);
        LuaMsg {
            ty: Config::TY_TIMER,
            receiver: service_id,
            data,
            ..Default::default()
        }
    }

    pub fn new_error(err: String, service_id: u32, session: i64) -> Self {
        let data = BinaryMut::new();
        LuaMsg {
            ty: Config::TY_ERROR,
            receiver: service_id,
            sessionid: session,
            err: Some(err),
            data,
            ..Default::default()
        }
    }

    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        LuaObject::<LuaMsg>::object_def(lua, "read_bool", hclua::function1(Self::read_bool));
        LuaObject::<LuaMsg>::object_def(lua, "read_u64", hclua::function1(Self::read_u64));
        LuaObject::<LuaMsg>::object_def(lua, "read_i64", hclua::function1(Self::read_i64));
        LuaObject::<LuaMsg>::object_def(lua, "read_f32", hclua::function1(Self::read_f32));
        LuaObject::<LuaMsg>::object_def(lua, "read_f64", hclua::function1(Self::read_f64));
        LuaObject::<LuaMsg>::object_def(lua, "read_str", hclua::function1(Self::read_str));

        LuaObject::<LuaMsg>::object_def(lua, "write_bool", hclua::function2(Self::write_bool));
        LuaObject::<LuaMsg>::object_def(lua, "write_u64", hclua::function2(Self::write_u64));
        LuaObject::<LuaMsg>::object_def(lua, "write_i64", hclua::function2(Self::write_i64));
        LuaObject::<LuaMsg>::object_def(lua, "write_f32", hclua::function2(Self::write_f32));
        LuaObject::<LuaMsg>::object_def(lua, "write_f64", hclua::function2(Self::write_f64));
        LuaObject::<LuaMsg>::object_def(lua, "write_str", hclua::function2(Self::write_str));
        LuaObject::<LuaMsg>::object_register(lua, "read_obj", Self::read_obj);
    }

    pub fn write_bool(&mut self, val: bool) {
        self.data.put_bool(val);
    }

    pub fn read_bool(&mut self) -> Option<bool> {
        self.data.try_get_bool().ok()
    }

    pub fn write_u64(&mut self, val: u64) {
        self.data.put_u64(val);
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        self.data.try_get_u64().ok()
    }

    pub fn write_i64(&mut self, val: i64) {
        self.data.put_i64(val);
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        self.data.try_get_i64().ok()
    }

    pub fn write_f32(&mut self, val: f32) {
        self.data.put_f32(val);
    }

    pub fn read_f32(&mut self) -> Option<f32> {
        self.data.try_get_f32().ok()
    }

    pub fn write_f64(&mut self, val: f64) {
        self.data.put_f64(val);
    }

    pub fn read_f64(&mut self) -> Option<f64> {
        self.data.try_get_f64().ok()
    }

    pub fn write_str(&mut self, val: String) {
        let _ = hcproto::encode_string(&mut self.data, &val);
    }

    pub fn read_str(&mut self) -> Option<String> {
        hcproto::decode_string(&mut self.data).ok()
    }

    extern "C" fn read_obj(lua: *mut lua_State) -> libc::c_int {
        let msg: &mut LuaMsg = unwrap_or!(hclua::read_wrapper_light_userdata(lua, 1), return 0);
        let obj = unwrap_or!(msg.obj.take(), return 0);
        obj.push_to_lua(lua)
    }
}
