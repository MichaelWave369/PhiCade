use crate::{ActionEnvelope, GameImage};

#[derive(Debug, Default)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct AudioBuffer {
    pub sample_rate_hz: u32,
    pub interleaved_stereo_f32: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    UnsupportedImage,
    MissingFirmware(String),
    InvalidState(String),
    Runtime(String),
}

pub trait EmulatorCore {
    fn core_id(&self) -> &str;
    fn load_game(&mut self, image: &GameImage) -> Result<(), CoreError>;
    fn reset(&mut self) -> Result<(), CoreError>;

    /// Advance exactly one emulated frame.
    ///
    /// Inputs are already authority-resolved Action Bus events. A core adapter
    /// must not fetch human, replay, script, or agent input by any side channel.
    fn step_frame(
        &mut self,
        actions: &[ActionEnvelope],
        video: &mut FrameBuffer,
        audio: &mut AudioBuffer,
    ) -> Result<(), CoreError>;
}
