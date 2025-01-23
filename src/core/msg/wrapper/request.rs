use hclua::{impl_obj_fn, Lua, LuaObject, LuaPush, ObjectMacro};
use webparse::Request;

#[derive(ObjectMacro)]
#[hclua_cfg(name = LuaMsg)]
#[hclua_cfg(light)]
pub struct WrapperRequest {
    #[hclua_skip]
    req: Request<Vec<u8>>,
}

impl Default for WrapperRequest {
    fn default() -> Self {
        let req = Request::builder().body(vec![]).unwrap();
        Self { req }
    }
}

impl WrapperRequest {
    pub fn new(req: Request<Vec<u8>>) -> Self {
        Self { req }
    }
    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);

        impl_obj_fn!(WrapperRequest, lua, req, is_http2);
    }
}
