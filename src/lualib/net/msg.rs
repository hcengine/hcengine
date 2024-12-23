use hclua::{Lua, LuaObject, ObjectMacro};
use hcnet::Message;

#[derive(ObjectMacro)]
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

    pub fn pack_text(text: String) -> Self {
        WrapMessage::new(Message::Text(text))
    }

    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        LuaObject::<WrapMessage>::object_def(lua, "get_type", hclua::function1(Self::get_type));
        LuaObject::<WrapMessage>::object_def(lua, "get_string", hclua::function1(Self::get_string));
        WrapMessage::object_static_def(lua, "pack_text", hclua::function1(Self::pack_text));
    }
}
