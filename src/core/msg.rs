use hcnet::Message;

use super::ServiceConf;


pub enum HcMsg {
    Msg(Message),
    NewService(ServiceConf),
    Stop(i32),
}