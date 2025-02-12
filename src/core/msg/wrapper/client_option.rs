use std::time::Duration;

use hclua::{Lua, ObjectMacro};
use webparse::{Url, WebError};
use wmhttp::{ClientOption, ProtError, ProxyScheme, TimeoutLayer};

#[derive(ObjectMacro, Default)]
#[hclua_cfg(name = ClientOption)]
#[hclua_cfg(light)]
pub struct WrapperClientOption {
    #[hclua_skip]
    pub client: ClientOption,
}

impl WrapperClientOption {
    
    pub fn new(client: ClientOption) -> Self {
        Self { client }
    }

    pub fn register_all(lua: &mut Lua) {
        Self::register_field(lua);

        type Object = WrapperClientOption;
        Object::object_def(lua, "http2", hclua::function1(Self::http2));
        Object::object_def(lua, "set_http2", hclua::function2(Self::set_http2));
        Object::object_def(lua, "http2_only", hclua::function1(Self::http2_only));
        Object::object_def(lua, "set_http2_only", hclua::function2(Self::set_http2_only));
        Object::object_def(lua, "timeout", hclua::function1(Self::timeout));
        Object::object_def(lua, "set_timeout", hclua::function2(Self::set_timeout));
        Object::object_def(lua, "connect_timeout", hclua::function1(Self::connect_timeout));
        Object::object_def(lua, "set_connect_timeout", hclua::function2(Self::set_connect_timeout));
        Object::object_def(lua, "ka_timeout", hclua::function1(Self::ka_timeout));
        Object::object_def(lua, "set_ka_timeout", hclua::function2(Self::set_ka_timeout));
        Object::object_def(lua, "read_timeout", hclua::function1(Self::read_timeout));
        Object::object_def(lua, "set_read_timeout", hclua::function2(Self::set_read_timeout));
        Object::object_def(lua, "write_timeout", hclua::function1(Self::write_timeout));
        Object::object_def(lua, "set_write_timeout", hclua::function2(Self::set_write_timeout));
        Object::object_def(lua, "add_proxy", hclua::function2(Self::add_proxy));
        Object::object_def(lua, "set_url", hclua::function2(Self::set_url));
    }

    pub fn http2(&mut self) -> bool {
        self.client.http2
    }
    
    pub fn set_http2(&mut self, http2: bool) {
        self.client.http2 = http2;
    }

    
    pub fn http2_only(&mut self) -> bool {
        self.client.http2_only
    }
    
    pub fn set_http2_only(&mut self, http2_only: bool) {
        self.client.http2_only = http2_only;
    }

    pub fn timeout(&self) -> u64 {
        if let Some(l) = &self.client.timeout {
            l.timeout.map(|d| d.as_millis() as u64).unwrap_or(0)
        } else {
            0
        }
    }
    
    pub fn set_timeout(&mut self, t: u64) {
        if let Some(l) = &mut self.client.timeout {
            l.timeout = Some(Duration::from_millis(t));
        } else {
            let mut tl = TimeoutLayer::new();
            tl.timeout = Some(Duration::from_millis(t));
            self.client.timeout = Some(tl);
        }
    }

    pub fn connect_timeout(&self) -> u64 {
        if let Some(l) = &self.client.timeout {
            l.connect_timeout.map(|d| d.as_millis() as u64).unwrap_or(0)
        } else {
            0
        }
    }
    
    pub fn set_connect_timeout(&mut self, t: u64) {
        if let Some(l) = &mut self.client.timeout {
            l.connect_timeout = Some(Duration::from_millis(t));
        } else {
            let mut tl = TimeoutLayer::new();
            tl.connect_timeout = Some(Duration::from_millis(t));
            self.client.timeout = Some(tl);
        }
    }

    
    pub fn ka_timeout(&self) -> u64 {
        if let Some(l) = &self.client.timeout {
            l.ka_timeout.map(|d| d.as_millis() as u64).unwrap_or(0)
        } else {
            0
        }
    }
    
    pub fn set_ka_timeout(&mut self, t: u64) {
        if let Some(l) = &mut self.client.timeout {
            l.ka_timeout = Some(Duration::from_millis(t));
        } else {
            let mut tl = TimeoutLayer::new();
            tl.ka_timeout = Some(Duration::from_millis(t));
            self.client.timeout = Some(tl);
        }
    }

    
    pub fn read_timeout(&self) -> u64 {
        if let Some(l) = &self.client.timeout {
            l.read_timeout.map(|d| d.as_millis() as u64).unwrap_or(0)
        } else {
            0
        }
    }
    
    pub fn set_read_timeout(&mut self, t: u64) {
        if let Some(l) = &mut self.client.timeout {
            l.read_timeout = Some(Duration::from_millis(t));
        } else {
            let mut tl = TimeoutLayer::new();
            tl.read_timeout = Some(Duration::from_millis(t));
            self.client.timeout = Some(tl);
        }
    }

    
    pub fn write_timeout(&self) -> u64 {
        if let Some(l) = &self.client.timeout {
            l.write_timeout.map(|d| d.as_millis() as u64).unwrap_or(0)
        } else {
            0
        }
    }
    
    pub fn set_write_timeout(&mut self, t: u64) {
        if let Some(l) = &mut self.client.timeout {
            l.write_timeout = Some(Duration::from_millis(t));
        } else {
            let mut tl = TimeoutLayer::new();
            tl.write_timeout = Some(Duration::from_millis(t));
            self.client.timeout = Some(tl);
        }
    }

    pub fn add_proxy(&mut self, proxy: String) -> Result<(), ProtError> {
        let proxy = ProxyScheme::try_from(&*proxy)?;
        self.client.proxies.push(proxy);
        Ok(())
    }

    pub fn set_url(&mut self, url: String) -> Result<(), WebError> {
        self.client.url = Some(Url::try_from(url)?);
        Ok(())
    }
}
