use std::{collections::HashMap, str::FromStr};

use hclua::{impl_obj_fn, Lua, LuaObject, LuaPush, ObjectMacro, RawString};
use webparse::{Method, Request, Scheme, Url, Version, WebError};
use wmhttp::{Body, RecvRequest};

#[derive(ObjectMacro, Debug)]
#[hclua_cfg(name = Request)]
#[hclua_cfg(light)]
pub struct WrapperRequest {
    #[hclua_skip]
    pub r: RecvRequest,
}

impl Default for WrapperRequest {
    fn default() -> Self {
        let r = Request::builder()
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:136.0) Gecko/20100101 Hcengine/1.0",
            )
            .body(Body::empty())
            .unwrap();
        Self { r }
    }
}

impl WrapperRequest {
    pub fn new(r: RecvRequest) -> Self {
        Self { r }
    }
    pub fn register_all(lua: &mut Lua) {
        Self::register(lua);

        type Object = WrapperRequest;
        impl_obj_fn!(Object, lua, r, is_http2);
        Object::object_def(lua, "method", hclua::function1(Self::method));
        Object::object_def(lua, "set_method", hclua::function2(Self::set_method));

        Object::object_def(lua, "set_url", hclua::function2(Self::set_url));
        Object::object_def(lua, "url", hclua::function1(Self::url));

        Object::object_def(lua, "set_username", hclua::function2(Self::set_username));
        Object::object_def(lua, "username", hclua::function1(Self::username));

        Object::object_def(lua, "set_password", hclua::function2(Self::set_password));
        Object::object_def(lua, "password", hclua::function1(Self::password));

        Object::object_def(lua, "set_domain", hclua::function2(Self::set_domain));
        Object::object_def(lua, "domain", hclua::function1(Self::domain));

        Object::object_def(lua, "set_query", hclua::function2(Self::set_query));
        Object::object_def(lua, "query", hclua::function1(Self::query));

        Object::object_def(lua, "set_port", hclua::function2(Self::set_port));
        Object::object_def(lua, "port", hclua::function1(Self::port));

        Object::object_def(lua, "set_version", hclua::function2(Self::set_version));

        Object::object_def(lua, "version", hclua::function1(Self::version));
        Object::object_def(lua, "set_body", hclua::function2(Self::set_body));
        Object::object_def(lua, "header_get", hclua::function2(Self::header_get));
        Object::object_def(lua, "header_set", hclua::function3(Self::header_set));
        Object::object_def(lua, "header_remove", hclua::function2(Self::header_remove));
        Object::object_def(lua, "header_clear", hclua::function1(Self::header_clear));
        Object::object_def(lua, "header_all", hclua::function1(Self::header_all));
    }

    pub fn method(&self) -> String {
        self.r.method().as_str().to_string()
    }

    pub fn set_method(&mut self, method: String) -> Result<(), WebError> {
        let method = Method::from_str(&method)?;
        self.r.set_method(method);
        Ok(())
    }

    pub fn set_version(&mut self, version: String) {
        self.r.set_version(Version::from(&*version));
    }

    pub fn url(&self) -> String {
        self.r.url().to_string()
    }

    pub fn set_url(&mut self, url: String) -> Result<(), WebError> {
        let url = Url::parse(url.into_bytes())?;
        self.r.set_url(url);
        Ok(())
    }

    pub fn username(&self) -> Option<String> {
        self.r.url().username.clone()
    }

    pub fn set_username(&mut self, username: String) {
        self.r.url_mut().username = Some(username);
    }

    pub fn password(&self) -> Option<String> {
        self.r.url().password.clone()
    }

    pub fn set_password(&mut self, password: String) {
        self.r.url_mut().password = Some(password);
    }

    pub fn domain(&self) -> Option<String> {
        self.r.url().domain.clone()
    }

    pub fn set_domain(&mut self, domain: String) {
        self.r.url_mut().domain = Some(domain);
    }

    pub fn query(&self) -> Option<String> {
        self.r.url().query.clone()
    }

    pub fn set_query(&mut self, query: String) {
        self.r.url_mut().query = Some(query);
    }

    pub fn port(&self) -> Option<u16> {
        self.r.url().port
    }

    pub fn set_port(&mut self, port: u16) {
        self.r.url_mut().port = Some(port);
    }

    pub fn path(&self) -> String {
        self.r.path().to_string()
    }

    pub fn set_path(&mut self, path: String) {
        self.r.set_path(path);
    }

    pub fn scheme(&self) -> String {
        self.r.scheme().as_str().to_string()
    }

    pub fn set_scheme(&mut self, scheme: String) {
        match Scheme::try_from(&*scheme) {
            Ok(s) => self.r.set_scheme(s),
            Err(_) => self.r.set_scheme(Scheme::Http),
        }
    }

    pub fn version(&self) -> &str {
        self.r.version().as_str()
    }

    pub fn set_body(&mut self, body: RawString) {
        self.r.body_mut().set_data(body.0);
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
