

#[derive(PartialEq, Eq)]
pub enum HcState {
    Init,
    Ready,
    Stopping,
    Stopped,
}