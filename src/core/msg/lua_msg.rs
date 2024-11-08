use algorithm::buf::BinaryMut;
use hclua::ObjectMacro;

#[derive(ObjectMacro, Default)]
#[hclua_cfg(name = LuaMsg)]
#[hclua_cfg(light)]
pub struct LuaMsg {
    #[hclua_field]
    pub ty: u8,
    #[hclua_field]
    pub sender: u32,
    #[hclua_field]
    pub receiver: u32,
    #[hclua_field]
    pub sessionid: i64,
    pub data: BinaryMut,
}