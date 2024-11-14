use std::str;
use hclua::{self, lua_State, LuaRead};
use hcproto::Value;
use std::collections::HashMap;
pub struct NetUtils;

impl NetUtils {
    pub fn lua_read_value(lua: *mut lua_State,
                          index: i32, stack: u32)
                          -> Option<Value> {
        if stack > 100 {
            return None
        };
        unsafe {
            let t = hclua::lua_type(lua, index);
            let value = match t {
                hclua::LUA_TBOOLEAN => {
                    let val: bool = unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
                    Some(Value::from(val))
                }
                hclua::LUA_TNUMBER => {
                    let val: f64 = unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
                    if val - val.floor() > 0.00001 {
                        Some(Value::from(val as f64))
                    } else {
                        Some(Value::from(val as i64))
                    }
                }
                hclua::LUA_TSTRING => {
                    let mut dst = unwrap_or!(LuaUtils::read_str_to_vec(lua, index), return None);

                    if dst.len() > 4 && dst[0] == 140 && dst[1] == 150 && dst[2] == 141 && dst[3] == 151 {
                        dst.drain(0..4);
                        return Some(Value::from(dst));
                    }
                    
                    if let Some(val) = str::from_utf8(&dst).ok() {
                        Some(Value::Str(val.to_string()))
                    } else {
                        Some(Value::from(dst))
                    }
                }
                hclua::LUA_TTABLE => {
                    if !hclua::lua_istable(lua, index) {
                        return None;
                    }
                    let len = hclua::lua_rawlen(lua, index);
                    if len > 0 {
                        let mut val: Vec<Value> = Vec::new();
                        for i in 1..(len + 1) {
                            hclua::lua_pushnumber(lua, i as f64);
                            let new_index = if index < 0 {
                                index - 1
                            } else {
                                index
                            };
                            hclua::lua_gettable(lua, new_index);
                            let sub_val = NetUtils::lua_read_value(lua,
                                                                    -1, stack + 1);
                            if sub_val.is_none() {
                                return None;
                            }
                            val.push(sub_val.unwrap());
                            hclua::lua_pop(lua, 1);
                        }
                        Some(Value::from(val))
                    } else {
                        let mut val: HashMap<Value, Value> = HashMap::new();
                        hclua::lua_pushnil(lua);
                        let t = if index < 0 {
                            index - 1
                        } else {
                            index
                        };

                        while hclua::lua_istable(lua, t) && hclua::lua_next(lua, t) != 0 {
                            let sub_val = unwrap_or!(NetUtils::lua_read_value(lua, -1, stack + 1), return None);
                            let value = if hclua::lua_isnumber(lua, -2) != 0 {
                                let idx: u32 = unwrap_or!(LuaRead::lua_read_at_position(lua, -2),
                                return None);
                                Value::from(idx)
                            } else {
                                let key: String = unwrap_or!(LuaRead::lua_read_at_position(lua, -2),
                                return None);
                                Value::from(key)
                            };
                            val.insert(value, sub_val);
                            hclua::lua_pop(lua, 1);
                        }
                        Some(Value::from(val))
                    }
                }
                _ => Some(Value::Nil),
            };
            value
        }
    }

    pub fn lua_convert_value(lua: *mut lua_State,
                             index: i32)
                             -> Option<Vec<Value>> {
        let size = unsafe { hclua::lua_gettop(lua) - index + 1 };
        let mut val: Vec<Value> = Vec::new();
        for i in 0..size {
            let sub_val = NetUtils::lua_read_value(lua,
                                                   i + index, 0);
            if sub_val.is_none() {
                return None;
            }
            val.push(sub_val.unwrap());
        }
        Some(val)
    }
}
