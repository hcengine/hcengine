use algorithm::buf::Bt;
use hclua::{self, Lua, LuaPush};
use hcproto::Buffer;
use log::warn;


use crate::{Config, LuaMsg};

use super::{LuaWrapperTableValue, SerUtils};

pub struct ProtoLua;

impl ProtoLua {
    pub fn pack_protocol(lua: *mut hclua::lua_State, index: i32) -> Option<LuaMsg> {
        let value = SerUtils::lua_convert_value(lua, index);
        if value.is_none() {
            warn!("pack_protocol failed");
            return None;
        }
        let value = value.unwrap();
        let mut buffer = Buffer::new();
        unwrap_or!(hcproto::encode_msg(&mut buffer, value).ok(), return None);
        if buffer.len() > 0xFFFFFF {
            println!("pack message(lua msg) size > 0xFFFF fail!");
            return None;
        }
        println!("buffer = {:?}", buffer.chunk());
        Some(LuaMsg::new(Config::TY_LUA, buffer.buf))
    }

    pub fn unpack_protocol(lua: *mut hclua::lua_State, msg: &mut LuaMsg) -> Option<i32> {
        println!("buffer = {:?}", msg.data.chunk());
        let mut buffer = Buffer::new_with(&mut msg.data);
        if let Ok(val) = hcproto::decode_msg(&mut buffer) {
            println!("ret val = {:?}", val);
            LuaWrapperTableValue(val).push_to_lua(lua);
            return Some(1);
        } else {
            return Some(0);
        }
    }

}
