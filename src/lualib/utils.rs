use std::ptr;

use hclua::lua_State;

struct LuaUtils;
impl LuaUtils {
    pub fn read_str_to_vec(lua: *mut lua_State, index: i32) -> Option<Vec<u8>> {
        let mut size: libc::size_t = 0;
        let c_str_raw = unsafe { hclua::lua_tolstring(lua, index, &mut size) };
        if c_str_raw.is_null() {
            return None;
        }
        unsafe {
            let mut dst: Vec<u8> = Vec::with_capacity(size);
            ptr::copy(c_str_raw as *mut u8, dst.as_mut_ptr(), size);
            dst.set_len(size);
            Some(dst)
        }
    }
}
