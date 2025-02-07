use hclua::{Lua, LuaObject, ObjectMacro};
use webparse::Response;
use wmhttp::{Body, RecvResponse};

#[derive(ObjectMacro)]
#[hclua_cfg(name = Response)]
#[hclua_cfg(light)]
pub struct WrapperResponse {
    #[hclua_skip]
    pub res: RecvResponse,
}

impl Default for WrapperResponse {
    fn default() -> Self {
        let res = Response::builder().body(Body::empty()).unwrap();
        Self { res }
    }
}

impl WrapperResponse {
    pub fn new(res: RecvResponse) -> Self {
        Self { res }
    }
    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);

        // impl_obj_fn!(WrapperResponse, lua, req, is_http2);
        LuaObject::<WrapperResponse>::object_def(lua, "set_text", hclua::function2(Self::set_text));
    }

    pub fn set_text(&mut self, text: String) {
        self.res.body_mut().set_text(text);
    }
}

