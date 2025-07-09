#[derive(Debug, Clone, PartialEq)]
pub enum EarCoverState {
    Both,
    Single,
    None,
}

#[derive(Debug, Copy, Clone)]
pub enum Anc {
    Off,
    NoiseCancelling,
    Transparency,
    Adaptive,
}
impl Anc {
    pub fn get_name(&self) -> &str {
        match self {
            Anc::Off => "Off",
            Anc::NoiseCancelling => "Noise Cancelling",
            Anc::Transparency => "Transparency",
            Anc::Adaptive => "Adaptive",
        }
    }
}
