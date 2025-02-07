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

        // impl_obj_fn!(WrapperResponse, lua, res, header);
        LuaObject::<WrapperResponse>::object_def(lua, "set_text", hclua::function2(Self::set_text));
        LuaObject::<WrapperResponse>::object_def(lua, "get_header", hclua::function2(Self::header_get));
        LuaObject::<WrapperResponse>::object_def(lua, "set_header", hclua::function3(Self::header_set));
    }

    pub fn set_text(&mut self, text: String) {
        self.res.body_mut().set_text(text);
    }

    pub fn header_get(&mut self, key: String) -> Option<String> {
        self.res.headers().get_str_value(&key)
    }

    pub fn get_host(&self) -> Option<String> {
        self.res.headers().get_host()
    }

    pub fn header_remove(&mut self, key: String) -> Option<String> {
        self.res.headers_mut().remove(&key).map(|v| v.to_string())
    }

    pub fn header_set(&mut self, key: String, val: String) {
        self.res.headers_mut().insert(key, val);
    }
}

