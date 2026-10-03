use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemId {
    Nes,
    Snes,
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    Genesis,
    Playstation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameImage {
    pub path: PathBuf,
    pub system: SystemId,
    pub display_name: String,
}

impl GameImage {
    pub fn new(path: impl Into<PathBuf>, system: SystemId, display_name: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            system,
            display_name: display_name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_image_keeps_user_selected_path() {
        let image = GameImage::new(
            "roms/homebrew-demo.nes",
            SystemId::Nes,
            "Homebrew Demo",
        );

        assert_eq!(image.path, PathBuf::from("roms/homebrew-demo.nes"));
        assert_eq!(image.system, SystemId::Nes);
    }
}
