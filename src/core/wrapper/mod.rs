use hclua::{impl_box_push, lua_State, LuaPush};
mod http;
mod redis;

pub use http::*;
pub use redis::*;

pub enum WrapperLuaMsg {
    Request(WrapperRequest),
    Response(WrapperResponse),
}

impl LuaPush for WrapperLuaMsg {
    fn push_to_lua(self, lua: *mut hclua::lua_State) -> i32 {
        ().push_to_lua(lua)
    }

    impl_box_push!();
}
