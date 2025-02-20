use hclua::{self, lua_State, LuaPush, LuaRead, LuaTable};
use redis::{Cmd, Msg, RedisError, RedisResult, Value};

use crate::LuaUtils;

static STATUS_SUFFIX: &'static str = "::STATUS";
static ERROR_SUFFIX: &'static str = "::ERROR";
static STATUS_TYPE: u32 = 1;
static ERROR_TYPE: u32 = 2;
/// the wrapper for push to lua
pub struct WrapperRedisValue(pub Value);
pub struct RedisWrapperError(pub RedisError);
pub struct RedisWrapperResult(pub RedisResult<Value>);
pub struct RedisWrapperMsg(pub Msg);

pub struct RedisWrapperVecVec(pub Vec<Vec<u8>>);
pub struct RedisWrapperCmd(pub Cmd);
pub struct RedisWrapperBatchVecVec(pub Vec<Vec<Vec<u8>>>);
pub struct RedisWrapperBatchCmd(pub Vec<Cmd>);

fn push_vec_value(lua: *mut lua_State, value: Vec<Value>) -> i32 {
    unsafe {
        hclua::lua_newtable(lua);

        for (i, v) in value.into_iter().enumerate() {
            i.push_to_lua(lua);
            let len = WrapperRedisValue(v).push_to_lua(lua);
            if len > 1 {
                hclua::lua_pop(lua, len - 1);
            }
            hclua::lua_settable(lua, -3);
        }
        1
    }
}

impl LuaPush for WrapperRedisValue {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self.0 {
            Value::Nil => ().push_to_lua(lua),
            Value::Int(val) => val.push_to_lua(lua),
            Value::BulkString(val) => {
                unsafe {
                    hclua::lua_pushlstring(lua, val.as_ptr() as *const libc::c_char, val.len())
                };
                1
            }
            Value::Array(val) => push_vec_value(lua, val),
            Value::SimpleString(val) => val.push_to_lua(lua),
            Value::Okay => {
                "OK".push_to_lua(lua);
                STATUS_TYPE.push_to_lua(lua);
                2
            }
            Value::Map(val) => unsafe {
                hclua::lua_newtable(lua);

                for (k, v) in val {
                    let len = WrapperRedisValue(k).push_to_lua(lua);
                    if len > 1 {
                        hclua::lua_pop(lua, len - 1);
                    }
                    let len = WrapperRedisValue(v).push_to_lua(lua);
                    if len > 1 {
                        hclua::lua_pop(lua, len - 1);
                    }
                    hclua::lua_settable(lua, -3);
                }
                1
            },
            Value::Attribute { data, attributes } => {
                todo!()
            }
            Value::Set(val) => push_vec_value(lua, val),
            Value::Double(val) => val.push_to_lua(lua),
            Value::Boolean(val) => val.push_to_lua(lua),
            Value::VerbatimString { format, text } => {
                todo!()
            }
            Value::BigNumber(val) => {
                // val.push_to_lua(lua)
                todo!()
            }
            Value::Push { kind, data } => unsafe {
                hclua::lua_newtable(lua);
                "kind".push_to_lua(lua);
                format!("{}", kind).push_to_lua(lua);
                hclua::lua_settable(lua, -3);
                "data".push_to_lua(lua);
                push_vec_value(lua, data);
                1
            },
            Value::ServerError(e) => {
                format!("{}", e.details().unwrap_or("error")).push_to_lua(lua);
                ERROR_TYPE.push_to_lua(lua);
                2
            }
            // TODO!
            _ => {
                // let val = val + STATUS_SUFFIX;
                ().push_to_lua(lua)
            }
        }
    }
}

impl LuaPush for RedisWrapperError {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        let desc = format!("{}", self.0).to_string() + ERROR_SUFFIX;
        desc.push_to_lua(lua)
    }
}

impl LuaPush for RedisWrapperResult {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self.0 {
            Ok(val) => WrapperRedisValue(val).push_to_lua(lua),
            Err(err) => RedisWrapperError(err).push_to_lua(lua),
        }
    }
}
fn read_at_index(cmd: &mut Cmd, lua: *mut lua_State, index: i32) -> Option<()> {
    unsafe {
        let t = hclua::lua_type(lua, index);
        match t {
            hclua::LUA_TBOOLEAN => {
                if let Some(v) = <bool as LuaRead>::lua_read_at_position(lua, index) {
                    if v {
                        cmd.arg("1");
                    } else {
                        cmd.arg("0");
                    }
                } else {
                    return None;
                }
            }
            hclua::LUA_TNUMBER => {
                if let Some(v) = <f64 as LuaRead>::lua_read_at_position(lua, index) {
                    cmd.arg(v);
                } else {
                    return None;
                }
            }
            hclua::LUA_TSTRING => {
                if let Some(v) = LuaUtils::read_str_to_vec(lua, index) {
                    cmd.arg(v);
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }
    Some(())
}

impl LuaRead for RedisWrapperCmd {
    fn lua_read_with_pop_impl(
        lua: *mut lua_State,
        index: i32,
        _pop: i32,
    ) -> Option<RedisWrapperCmd> {
        let args = unsafe { hclua::lua_gettop(lua) - index.abs() + 1 };
        if args < 0 {
            return None;
        }
        let mut cmd = Cmd::new();
        for i in 0..args {
            unwrap_or!(read_at_index(&mut cmd, lua, i + index), return None);
        }
        Some(RedisWrapperCmd(cmd))
    }
}


impl LuaRead for RedisWrapperBatchCmd {
    fn lua_read_with_pop_impl(
        lua: *mut lua_State,
        index: i32,
        _pop: i32,
    ) -> Option<RedisWrapperBatchCmd> {
        let args = unsafe { hclua::lua_gettop(lua) - index.abs() + 1 };
        let mut vecs = vec![];
        if args < 0 {
            return None;
        }
        unsafe {
            for i in 0..args {
                let new_idx = i + index;
                if !hclua::lua_istable(lua, new_idx) {
                    return None;
                }
                let len = hclua::lua_rawlen(lua, new_idx);
                for j in 1..len + 1 {
                    j.push_to_lua(lua);
                    hclua::lua_gettable(lua, new_idx);
                    let mut cmd = Cmd::new();
                    unwrap_or!(read_at_index(&mut cmd, lua, -1), {
                        hclua::lua_pop(lua, 1);
                        return None;
                    });
                    vecs.push(cmd);
                }
                hclua::lua_pop(lua, 1);
            }
        }
        Some(RedisWrapperBatchCmd(vecs))
    }
}
