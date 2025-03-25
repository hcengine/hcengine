use std::collections::HashMap;
use hclua::{Lua, LuaObject, ObjectMacro, RawString};
use webparse::{Response, StatusCode, WebError};
use wmhttp::{Body, RecvResponse};

#[derive(ObjectMacro, Debug)]
#[hclua_cfg(name = Response)]
pub struct WrapperResponse {
    #[hclua_skip]
    pub r: RecvResponse,
    #[hclua_skip]
    pub body: Option<Vec<u8>>,
}

impl Default for WrapperResponse {
    fn default() -> Self {
        let r = Response::builder().body(Body::empty()).unwrap();
        Self { r, body: None }
    }
}

impl WrapperResponse {
    pub fn new(r: RecvResponse) -> Self {
        Self { r, body: None }
    }
    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);
        type Object = WrapperResponse;
        Object::object_def(lua, "status_code", hclua::function1(Self::status_code));
        Object::object_def(lua, "set_status_code", hclua::function2(Self::set_status_code));
        Object::object_def(lua, "status_str", hclua::function1(Self::status_str));
        
        Object::object_def(lua, "version", hclua::function1(Self::version));
        Object::object_def(lua, "set_body", hclua::function2(Self::set_body));
        Object::object_def(lua, "get_body", hclua::function1(Self::get_body));
        Object::object_def(lua, "header_get", hclua::function2(Self::header_get));
        Object::object_def(lua, "header_set", hclua::function3(Self::header_set));
        Object::object_def(lua, "header_remove", hclua::function2(Self::header_remove));
        Object::object_def(lua, "header_clear", hclua::function1(Self::header_clear));
        Object::object_def(lua, "header_all", hclua::function1(Self::header_all));
    }


    pub fn status_code(&self) -> u16 {
        self.r.status().as_u16()
    }
    
    pub fn set_status_code(&mut self, code: u16) -> Result<(), WebError> {
        *self.r.status_mut() = StatusCode::from_u16(code)?;
        Ok(())
    }
    
    pub fn status_str(&self) -> &str {
        self.r.status().as_str()
    }

    pub fn version(&self) -> &str {
        self.r.version().as_str()
    }

    pub fn set_body(&mut self, body: RawString) {
        self.r.body_mut().set_data(body.0);
    }

    pub fn get_body(&mut self) -> Option<RawString> {
        if self.body.is_none() {
            let bin = self.r.body_mut().read_now();
            let v = bin.into_slice_all();
            self.body = Some(v);
        }
        return self.body.clone().map(|v| RawString(v))
    }

    pub fn get_host(&self) -> Option<String> {
        self.r.headers().get_host()
    }

    pub fn header_get(&mut self, key: String) -> Option<String> {
        self.r.headers().get_str_value(&key)
    }

    pub fn header_set(&mut self, key: String, val: String) {
        self.r.headers_mut().insert(key, val);
    }
    
    pub fn header_remove(&mut self, key: String) -> Option<String> {
        self.r.headers_mut().remove(&key).map(|v| v.to_string())
    }
    
    pub fn header_all(&mut self) -> HashMap<String, String> {
        let mut ret = HashMap::new();
        for (k, v) in self.r.headers().iter() {
            ret.insert(k.to_string(), v.to_string());
        }
        ret
    }


    pub fn header_clear(&mut self) {
        self.r.headers_mut().clear();
    }
}
