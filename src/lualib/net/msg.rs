use hclua::{Lua, LuaObject, ObjectMacro};
use hcnet::Message;

#[derive(ObjectMacro)]
pub struct WarpMessage {
    #[hclua_skip]
    pub msg: Message,
}

impl Default for WarpMessage {
    fn default() -> Self {
        Self {
            msg: Message::Shutdown,
        }
    }
}

impl WarpMessage {
    pub fn new(msg: Message) -> Self {
        Self { msg }
    }


    pub fn get_type(&self) -> u8 {
        
    }
    
    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        LuaObject::<WarpMessage>::object_def(lua, "read_bool", hclua::function1(Self::read_bool));
    }
}
