pub struct State {
    pub volume: f32,
    pub power: Power,
    pub input: Input,
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Eq)]
pub enum Power {
    On,
    Off,
}

#[derive(Debug, defmt::Format, Clone, PartialEq, Eq)]
pub enum Input {
    Optical,
    Wired,
    AirPlay,
    Unknown,
}
