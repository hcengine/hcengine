use hclua::{self, lua_State, LuaPush, LuaRead, LuaTable};
use redis::{Cmd, Msg, RedisError, RedisResult, Value};

static STATUS_SUFFIX: &'static str = "::STATUS";
static ERROR_SUFFIX: &'static str = "::ERROR";
/// the wrapper for push to lua
pub struct RedisWrapperValue(pub Value);
pub struct RedisWrapperError(pub RedisError);
pub struct RedisWrapperResult(pub RedisResult<Value>);
pub struct RedisWrapperMsg(pub Msg);

pub struct RedisWrapperVecVec(pub Vec<Vec<u8>>);
pub struct RedisWrapperCmd(pub Cmd);
pub struct RedisWrapperBatchVecVec(pub Vec<Vec<Vec<u8>>>);
pub struct RedisWrapperBatchCmd(pub Vec<Cmd>);

impl LuaPush for RedisWrapperValue {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self.0 {
            Value::Nil => ().push_to_lua(lua),
            Value::Int(val) => (val as u32).push_to_lua(lua),
            Value::BulkString(val) => {
                unsafe {
                    hclua::lua_pushlstring(lua, val.as_ptr() as *const libc::c_char, val.len())
                };
                1
            }
            Value::Array(mut val) => {
                let mut wrapper_val: Vec<RedisWrapperValue> = vec![];
                for v in val.drain(..) {
                    wrapper_val.push(RedisWrapperValue(v));
                }
                wrapper_val.push_to_lua(lua)
            }
            Value::Okay => {
                let val = "OK".to_string() + STATUS_SUFFIX;
                val.push_to_lua(lua)
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
            Ok(val) => RedisWrapperValue(val).push_to_lua(lua),
            Err(err) => RedisWrapperError(err).push_to_lua(lua),
        }
    }
}

impl LuaPush for RedisWrapperMsg {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe {
            hclua::lua_newtable(lua);

            let payload: RedisResult<Value> = self.0.get_payload();
            if payload.is_ok() {
                "payload".push_to_lua(lua);
                RedisWrapperValue(payload.ok().unwrap()).push_to_lua(lua);
                hclua::lua_settable(lua, -3);
            }

            "channel".push_to_lua(lua);
            self.0.get_channel_name().push_to_lua(lua);
            hclua::lua_settable(lua, -3);

            let pattern: RedisResult<String> = self.0.get_pattern();
            if pattern.is_ok() {
                "pattern".push_to_lua(lua);
                pattern.ok().unwrap().push_to_lua(lua);
                hclua::lua_settable(lua, -3);
            }
            1
        }
    }
}

impl LuaRead for RedisWrapperVecVec {
    fn lua_read_with_pop_impl(
        lua: *mut lua_State,
        index: i32,
        _pop: i32,
    ) -> Option<RedisWrapperVecVec> {
        let args = unsafe { hclua::lua_gettop(lua) - index.abs() + 1 };
        let mut vecs = vec![];
        if args < 0 {
            return None;
        }
        for i in 0..args {
            let mut val: Option<Vec<u8>> = None;
            let bval: Option<bool> = LuaRead::lua_read_at_position(lua, i + index);
            if let Some(b) = bval {
                if b {
                    val = Some("1".to_string().into_bytes());
                } else {
                    val = Some("0".to_string().into_bytes());
                }
            }
            // if val.is_none() {
            //     let dst = unwrap_or!(LuaUtils::read_str_to_vec(lua, i + index), return None);
            //     val = Some(dst);
            // }
            if val.is_none() {
                return None;
            }
            vecs.push(val.unwrap());
        }
        Some(RedisWrapperVecVec(vecs))
    }
}

impl LuaRead for RedisWrapperBatchVecVec {
    fn lua_read_with_pop_impl(
        lua: *mut lua_State,
        index: i32,
        _pop: i32,
    ) -> Option<RedisWrapperBatchVecVec> {
        let args = unsafe { hclua::lua_gettop(lua) - index.abs() + 1 };
        let mut vecs = vec![];
        if args < 0 {
            return None;
        }
        for i in 0..args {
            let mut table: LuaTable =
                unwrap_or!(LuaRead::lua_read_at_position(lua, i + index), return None);
            let mut sub_vec = vec![];
            for j in 1..table.table_len() + 1 {
                let val: Option<String> = table.query(j);
                if val.is_some() {
                    sub_vec.push(val.unwrap().into_bytes());
                    continue;
                }
                let val: Option<f32> = table.query(j);
                if val.is_some() {
                    sub_vec.push(format!("{}", val.unwrap()).into_bytes());
                    continue;
                } else {
                    return None;
                }
            }
            vecs.push(sub_vec);
        }
        Some(RedisWrapperBatchVecVec(vecs))
    }
}

impl LuaRead for RedisWrapperCmd {
    fn lua_read_with_pop_impl(
        lua: *mut lua_State,
        index: i32,
        _pop: i32,
    ) -> Option<RedisWrapperCmd> {
        let vecs: RedisWrapperVecVec =
            unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
        let mut cmd = Cmd::new();
        for vec in vecs.0 {
            cmd.arg(vec);
        }

        // ObjectMgr::instance().obj_alloc("RedisWrapperCmd".to_string());
        Some(RedisWrapperCmd(cmd))
    }
}

impl Drop for RedisWrapperCmd {
    fn drop(&mut self) {
        // ObjectMgr::instance().obj_dealloc("RedisWrapperCmd".to_string());
    }
}

impl LuaRead for RedisWrapperBatchCmd {
    fn lua_read_with_pop_impl(
        lua: *mut lua_State,
        index: i32,
        _pop: i32,
    ) -> Option<RedisWrapperBatchCmd> {
        let vecs: RedisWrapperBatchVecVec =
            unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
        let mut cmds = vec![];
        for vec in vecs.0 {
            let mut cmd = Cmd::new();
            cmd.arg(vec);
            cmds.push(cmd);
        }
        Some(RedisWrapperBatchCmd(cmds))
    }
}
