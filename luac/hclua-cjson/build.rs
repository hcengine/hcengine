fn main() {
    cc::Build::new()
        .file("cjson/fpconv.c")
        .file("cjson/lua_cjson.c")
        .file("cjson/strbuf.c")
        .include("cjson")
        .include("include")
        .compile("libhclua-cjson.a");
}
