

#[derive(PartialEq, Eq)]
pub enum HcStatusState {
    Init,
    Ready,
    Stopping,
    Stopped,
}