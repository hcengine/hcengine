use hclua::{impl_box_push, lua_State, LuaPush};
mod http;
mod redis;

pub use http::*;
use ::redis::Value;
pub use redis::*;
use wmhttp::{RecvRequest, RecvResponse};

#[derive(Debug)]
pub enum WrapperLuaMsg {
    Request(WrapperRequest),
    Response(WrapperResponse),
    Redis(WrapperRedisValue),
}

impl WrapperLuaMsg {
    pub fn request(r: RecvRequest) -> Self {
        Self::Request(WrapperRequest::new(r))
    }
    
    pub fn response(r: RecvResponse) -> Self {
        Self::Response(WrapperResponse::new(r))
    }
    
    pub fn redis(r: Value) -> Self {
        Self::Redis(WrapperRedisValue(r))
    }
}

impl LuaPush for WrapperLuaMsg {
    fn push_to_lua(self, lua: *mut hclua::lua_State) -> i32 {
        match self {
            WrapperLuaMsg::Request(v) => v.push_to_lua(lua),
            WrapperLuaMsg::Response(v) => v.push_to_lua(lua),
            WrapperLuaMsg::Redis(v) => v.push_to_lua(lua),
        }
    }

    impl_box_push!();
}
