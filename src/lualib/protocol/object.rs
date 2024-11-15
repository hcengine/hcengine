use hclua::{lua_State, Lua, LuaObject, LuaPush, LuaRead, ObjectMacro};

use crate::LuaMsg;

use super::ProtoLua;


#[derive(ObjectMacro, Default)]
#[hclua_cfg(name = Protocol)]
pub struct ProtocolObject;

impl ProtocolObject {

    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        ProtocolObject::object_static_register(lua, "lua_pack", Self::lua_pack);
        ProtocolObject::object_static_register(lua, "lua_unpack", Self::lua_unpack);
    }


    extern "C" fn lua_pack(lua: *mut lua_State) -> libc::c_int {
        let msg = ProtoLua::pack_protocol(lua, 1);
        msg.push_to_lua(lua)
    }
    
    extern "C" fn lua_unpack(lua: *mut lua_State) -> libc::c_int {
        let msg: Option<&mut LuaMsg> = LuaRead::lua_read(lua);
        if let Some(v) = msg {
            ProtoLua::unpack_protocol(lua, v).unwrap_or(0)
        } else {
            0
        }
    }
}