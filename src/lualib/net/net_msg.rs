use std::mem;

use hclua::{Lua, LuaObject, ObjectMacro, RawString};
use hcnet::Message;

#[derive(ObjectMacro, Clone)]
#[hclua_cfg(name = NetMsg)]
#[hclua_cfg(light)]
pub struct WrapMessage {
    #[hclua_skip]
    pub msg: Message,
}

impl Default for WrapMessage {
    fn default() -> Self {
        Self {
            msg: Message::Shutdown,
        }
    }
}

impl WrapMessage {
    pub fn new(msg: Message) -> Self {
        Self { msg }
    }

    pub fn get_type(&self) -> u8 {
        self.msg.get_type()
    }

    pub fn get_string(&self) -> Option<&String> {
        match &self.msg {
            Message::Text(v) => return Some(v),
            _ => return None,
        }
    }

    pub fn get_lstring(&self) -> Option<RawString> {
        match &self.msg {
            Message::Text(v) => return Some(RawString(v.as_bytes().to_vec())),
            Message::Binary(v) => return Some(RawString(v.clone())),
            Message::Ping(v) => return Some(RawString(v.clone())),
            Message::Pong(v) => return Some(RawString(v.clone())),
            _ => return None,
        }
    }

    pub fn take_data(&mut self) -> Option<RawString> {
        let mut msg = Message::Unvaid;
        mem::swap(&mut msg, &mut self.msg);
        let data = match msg {
            Message::Text(v) => v.into_bytes(),
            Message::Binary(v) => v.clone(),
            Message::Ping(v) => v.clone(),
            Message::Pong(v) => v.clone(),
            _ => {
                return None;
            }
        };
        return Some(RawString(data));
    }

    pub fn clone_msg(&self) -> Self {
        self.clone()
    }

    pub fn pack_text(text: String) -> Self {
        WrapMessage::new(Message::Text(text))
    }

    pub fn pack_binary(raw: RawString) -> Self {
        WrapMessage::new(Message::Binary(raw.0))
    }

    pub fn pack_ping(raw: RawString) -> Self {
        WrapMessage::new(Message::Ping(raw.0))
    }

    pub fn pack_pong(raw: RawString) -> Self {
        WrapMessage::new(Message::Pong(raw.0))
    }

    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        WrapMessage::object_def(lua, "get_type", hclua::function1(Self::get_type));
        WrapMessage::object_def(lua, "get_string", hclua::function1(Self::get_string));
        WrapMessage::object_def(lua, "get_lstring", hclua::function1(Self::get_lstring));
        WrapMessage::object_def(lua, "take_data", hclua::function1(Self::take_data));
        WrapMessage::object_def(lua, "clone_msg", hclua::function1(Self::clone_msg));
        WrapMessage::object_static_def(lua, "pack_text", hclua::function1(Self::pack_text));
        WrapMessage::object_static_def(lua, "pack_binary", hclua::function1(Self::pack_binary));
        WrapMessage::object_static_def(lua, "pack_ping", hclua::function1(Self::pack_ping));
        WrapMessage::object_static_def(lua, "pack_pong", hclua::function1(Self::pack_pong));
    }
}
