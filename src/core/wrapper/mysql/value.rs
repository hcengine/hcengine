use std::sync::Arc;

use mysql_async::{Column, Row, Value};



#[derive(Debug)]
pub enum MysqlValue {
    /// 包含插入及更新数
    Only(Value),
    First(Option<Value>),
    Row(Option<Row>),
    ColRows(Vec<Row>),
    Col(Arc<[Column]>),
    Iter(Row),
    IterEnd,
}