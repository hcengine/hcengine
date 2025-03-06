
use hclua::{lua_State, LuaPush, LuaTable};
use mysql_async::Value;

mod value;

pub use value::MysqlValue;

#[derive(Debug)]
pub struct WrapperMysqlValue(pub MysqlValue);

#[derive(Debug)]
struct InnerWrapperMysqlValue(pub Value);

impl LuaPush for WrapperMysqlValue {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self.0 {
            MysqlValue::Only(value) => InnerWrapperMysqlValue(value).push_to_lua(lua),
            MysqlValue::First(value) => InnerWrapperMysqlValue(value.unwrap_or(Value::NULL)).push_to_lua(lua),
            MysqlValue::Row(row) => {
                if let Some(mut row) = row {
                    let mut table = LuaTable::create_table(lua);
                    for i in 0..row.len() {
                        table.insert(InnerWrapperMysqlValue(row.take(i).unwrap_or(Value::NULL)));
                    }
                } else {
                    ().push_to_lua(lua);
                }
                1
            },
            MysqlValue::Col(items) => {
                let mut table = LuaTable::create_table(lua);
                for v in items.into_iter() {
                    table.insert(&*v.name_str());
                }
                1
            },
            MysqlValue::ColRows(rows) => {
                let mut table = LuaTable::create_table(lua);
                {
                    let mut col_table = table.empty_table("cols");
                    if rows.len() > 0 {
                        let cols = rows[0].columns_ref();
                        for v in cols {
                            col_table.insert(&*v.name_str());
                        }
                    }
                }
                {
                    let mut rows_table = table.empty_table("rows");
                    for (i, mut val) in rows.into_iter().enumerate() {
                        let mut row = rows_table.empty_table(i + 1);
                        for i in 0..val.len() {
                            row.insert(InnerWrapperMysqlValue(val.take(i).unwrap_or(Value::NULL)));
                        }
                    }
                }
                1
            },
            MysqlValue::Iter(mut val) => {
                let mut table = LuaTable::create_table(lua);
                for i in 0..val.len() {
                    table.insert(InnerWrapperMysqlValue(val.take(i).unwrap_or(Value::NULL)));
                }
                1
            },
            MysqlValue::IterEnd => {
                ().push_to_lua(lua);
                1
            },
        }
    }
}


impl LuaPush for InnerWrapperMysqlValue {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self.0 {
            Value::NULL => {
                unsafe {
                    hclua::lua_pushnil(lua);
                }
                1
            },
            Value::Bytes(items) => {
                unsafe {
                    hclua::lua_pushlstring(lua, items.as_ptr() as *const libc::c_char, items.len())
                };
                1
            },
            Value::Int(v) => {
                v.push_to_lua(lua)
            },
            Value::UInt(v) => {
                v.push_to_lua(lua)
            },
            Value::Float(v) => {
                v.push_to_lua(lua)
            },
            Value::Double(v) => {
                v.push_to_lua(lua)
            },
            Value::Date(y, m, d, h, min, s, ms) => {
                let mut table = LuaTable::create_table(lua);
                table.set("year", 1900+y);
                table.set("month", 1+m);
                table.set("day", d);
                table.set("hour", h);
                table.set("min", min);
                table.set("sec", s);
                table.set("msec", ms);
                1
            },
            // is negative, days, hours, minutes, seconds, micro seconds
            Value::Time(is_neg, d, h, min, s, ms) => {
                let mut table = LuaTable::create_table(lua);
                table.set("is_neg", is_neg);
                table.set("day", d);
                table.set("hour", h);
                table.set("min", min);
                table.set("sec", s);
                table.set("msec", ms);
                1
            },
        }
    }
}