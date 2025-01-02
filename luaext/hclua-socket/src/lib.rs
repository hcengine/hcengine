
#[allow(improper_ctypes)]
extern "C" {
    pub fn luaopen_socket_core(L : *mut hclua::lua_State) -> libc::c_int;
}

extern "C" fn safe_luaopen_socket_core(lua: *mut hclua::lua_State) -> libc::c_int {
    unsafe {
        luaopen_socket_core(lua)
    }
}

/// custom lua load func
extern "C" fn load_func(lua: *mut hclua::lua_State) -> libc::c_int {
    let path:String = hclua::LuaRead::lua_read(lua).unwrap_or(String::new());
    if &path == "socket.core" || &path == "luasocket" || &path == "socket" {
        unsafe {
            hclua::lua_pushcfunction(lua, safe_luaopen_socket_core);
        }
        return 1;
    }
    return 0;
}

pub fn enable_socket_core(lua : &mut hclua::Lua) {
    lua.add_lualoader(load_func);
}