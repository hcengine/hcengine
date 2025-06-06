use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use des::Des;
use hclua::{Lua, RawString};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use sha2::Sha256;

type DesCbc = Cbc<Des, Pkcs7>;
type HmacSha256 = Hmac<Sha256>;
type HmacMd5 = Hmac<Md5>;

fn md5(val: RawString) -> String {
    let mut hasher = Md5::new();
    hasher.update(val.0);
    let result = hasher.finalize();
    let val = format!("{:x}", result);
    println!("MD5: {:x}", result);
    val
}

fn hmac_md5(secret_key: String, val: RawString) -> String {
    let mut mac = HmacMd5::new_from_slice(secret_key.as_bytes()).unwrap();
    mac.update(&val.0[..]);
    let result = mac.finalize().into_bytes();
    println!("HMAC-MD5: {:x}", result);
    let val = format!("{:x}", result);
    val
}

fn hmac_sha256(secret_key: String, val: RawString) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).unwrap();
    mac.update(&val.0[..]);
    let result = mac.finalize().into_bytes();
    println!("HMAC-MD5: {:x}", result);
    let val = format!("{:x}", result);
    val
}

fn base64_encode(val: RawString) -> String {
    use base64::{engine::general_purpose, Engine as _};

    // 标准 Base64 编码（带填充）
    let encoded = general_purpose::STANDARD.encode(val.0);
    println!("Encoded: {}", encoded); // SGVsbG8sIFJ1c3Qh
    encoded
}

fn base64_decode(val: String) -> Option<RawString> {
    use base64::{engine::general_purpose, Engine as _};
    // 解码
    match general_purpose::STANDARD.decode(&val) {
        Ok(d) => Some(RawString(d)),
        Err(_) => None,
    }
}

fn des_encode(key: String, iv: String, val: RawString) -> RawString {
    let cipher = DesCbc::new_from_slices(key.as_bytes(), iv.as_bytes()).unwrap();
    let ciphertext = cipher.encrypt_vec(&val.0);
    RawString(ciphertext)
}

fn des_decode(key: String, iv: String, val: RawString) -> Option<RawString> {
    let cipher = DesCbc::new_from_slices(key.as_bytes(), iv.as_bytes()).unwrap();
    match cipher.decrypt_vec(&val.0) {
        Ok(v) => Some(RawString(v)),
        Err(_) => None,
    }
}

#[hclua::lua_module(name = "crypt")]
fn crypt_module(lua: &mut Lua) -> libc::c_int {
    let mut table = lua.create_table();
    table.set("md5", hclua::function1(md5));
    table.set("hmac_md5", hclua::function2(hmac_md5));
    table.set("hmac_sha256", hclua::function2(hmac_sha256));
    table.set("base64_encode", hclua::function1(base64_encode));
    table.set("base64_decode", hclua::function1(base64_decode));
    table.set("des_encode", hclua::function3(des_encode));
    table.set("des_decode", hclua::function3(des_decode));
    1
}
