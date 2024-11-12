use algorithm::buf::BinaryMut;
use hclua::ObjectMacro;

#[derive(Default, ObjectMacro)]
#[hclua_cfg(name = LuaMsg)]
#[hclua_cfg(light)]
pub struct LuaMsg {
    pub ty: u8,
    pub sender: u32,
    pub receiver: u32,
    pub sessionid: i64,
    #[hclua_skip]
    pub data: BinaryMut,
}