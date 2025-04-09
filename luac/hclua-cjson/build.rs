fn main() {
    let mut build = cc::Build::new();

    build
        .file("cjson/fpconv.c")
        .file("cjson/lua_cjson.c")
        .file("cjson/strbuf.c")
        .include("cjson")
        .include("include");

    // #[cfg(feature = "lua51")]
    // build.define("LUA_VERSION_NUM", "501");
    // // #[cfg(feature = "lua51")]
    // // unreachable!();

    // #[cfg(feature = "lua52")]
    // build.define("LUA_VERSION_NUM", "502");
    // #[cfg(feature = "lua53")]
    // build.define("LUA_VERSION_NUM", "503");
    // #[cfg(feature = "lua54")]
    // build.define("LUA_VERSION_NUM", "504");

    build.compile("libhclua-cjson.a");
}
