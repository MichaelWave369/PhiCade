use crate::{
    ActionEnvelope, ContentDescriptor, GameImage, RuntimeCapabilities, RuntimeCapabilityManifest,
    RuntimeExecutionModel,
};

#[derive(Debug, Clone, Default)]
pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
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

    /// Describe what this runtime adapter can actually provide.
    ///
    /// The default is deliberately conservative: it only claims behavior
    /// guaranteed by the EmulatorCore trait itself. Adapters must opt into
    /// stronger capabilities, and QUALIFIED claims still require matching
    /// evidence for the exact runtime binary/profile.
    fn capability_manifest(&self) -> RuntimeCapabilityManifest {
        RuntimeCapabilityManifest::new(
            self.core_id(),
            None,
            "phicade.emulator-core",
            RuntimeExecutionModel::EmbeddedFrameCore,
            RuntimeCapabilities::core_baseline(),
        )
    }
    fn load_game(&mut self, image: &GameImage) -> Result<(), CoreError>;

    /// Load runtime-neutral content through the legacy GameImage compatibility
    /// bridge when possible. Non-file or non-system content is refused by
    /// emulator cores until a provider supplies a stronger content path.
    fn load_content(&mut self, content: &ContentDescriptor) -> Result<(), CoreError> {
        content.validate().map_err(CoreError::InvalidState)?;
        let image = content.as_game_image().ok_or(CoreError::UnsupportedImage)?;
        self.load_game(&image)
    }
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
