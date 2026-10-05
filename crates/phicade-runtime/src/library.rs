use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CONTENT_DESCRIPTOR_SCHEMA: &str = "phicade.content-descriptor.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SystemId {
    Nes,
    Snes,
    GameBoy,
    GameBoyColor,
    GameBoyAdvance,
    Genesis,
    Playstation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentLocator {
    File { path: PathBuf },
    Directory { path: PathBuf },
    LaunchTarget { target: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentDescriptor {
    pub schema: String,
    pub display_name: String,
    pub locator: ContentLocator,
    pub system: Option<SystemId>,
    pub runtime_hint: Option<String>,
}

impl ContentDescriptor {
    pub fn file(path: impl Into<PathBuf>, display_name: impl Into<String>) -> Self {
        Self {
            schema: CONTENT_DESCRIPTOR_SCHEMA.into(),
            display_name: display_name.into(),
            locator: ContentLocator::File { path: path.into() },
            system: None,
            runtime_hint: None,
        }
    }

    pub fn directory(path: impl Into<PathBuf>, display_name: impl Into<String>) -> Self {
        Self {
            schema: CONTENT_DESCRIPTOR_SCHEMA.into(),
            display_name: display_name.into(),
            locator: ContentLocator::Directory { path: path.into() },
            system: None,
            runtime_hint: None,
        }
    }

    pub fn launch_target(target: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            schema: CONTENT_DESCRIPTOR_SCHEMA.into(),
            display_name: display_name.into(),
            locator: ContentLocator::LaunchTarget {
                target: target.into(),
            },
            system: None,
            runtime_hint: None,
        }
    }

    pub fn with_system(mut self, system: SystemId) -> Self {
        self.system = Some(system);
        self
    }

    pub fn with_runtime_hint(mut self, runtime_hint: impl Into<String>) -> Self {
        self.runtime_hint = Some(runtime_hint.into());
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != CONTENT_DESCRIPTOR_SCHEMA {
            return Err(format!("unsupported content descriptor schema: {}", self.schema));
        }
        if self.display_name.trim().is_empty() {
            return Err("content display name must not be empty".into());
        }
        if self
            .runtime_hint
            .as_deref()
            .is_some_and(|hint| hint.trim().is_empty())
        {
            return Err("runtime hint must not be empty when present".into());
        }
        match &self.locator {
            ContentLocator::File { path } | ContentLocator::Directory { path } => {
                if path.as_os_str().is_empty() {
                    return Err("content path must not be empty".into());
                }
            }
            ContentLocator::LaunchTarget { target } => {
                if target.trim().is_empty() {
                    return Err("launch target must not be empty".into());
                }
            }
        }
        Ok(())
    }

    pub fn as_game_image(&self) -> Option<GameImage> {
        let ContentLocator::File { path } = &self.locator else {
            return None;
        };
        let system = self.system?;
        Some(GameImage::new(
            path.clone(),
            system,
            self.display_name.clone(),
        ))
    }
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

    pub fn as_content_descriptor(&self) -> ContentDescriptor {
        ContentDescriptor::from(self)
    }
}

impl From<&GameImage> for ContentDescriptor {
    fn from(image: &GameImage) -> Self {
        ContentDescriptor::file(image.path.clone(), image.display_name.clone())
            .with_system(image.system)
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

    #[test]
    fn legacy_game_image_round_trips_through_content_descriptor() {
        let image = GameImage::new(
            "roms/demo.gb",
            SystemId::GameBoy,
            "Demo",
        );
        let content = image.as_content_descriptor();
        content.validate().expect("valid descriptor");

        assert_eq!(content.system, Some(SystemId::GameBoy));
        assert_eq!(content.runtime_hint, None);
        assert_eq!(content.as_game_image(), Some(image));
    }

    #[test]
    fn directory_and_launch_target_are_not_forced_into_game_image() {
        let directory = ContentDescriptor::directory("games/monkey-island", "Monkey Island")
            .with_runtime_hint("scummvm");
        let target = ContentDescriptor::launch_target("monkey", "Monkey Island")
            .with_runtime_hint("scummvm");

        directory.validate().expect("valid directory");
        target.validate().expect("valid target");
        assert!(directory.as_game_image().is_none());
        assert!(target.as_game_image().is_none());
    }

    #[test]
    fn descriptor_serializes_stable_locator_shape() {
        let content = ContentDescriptor::file("games/demo.scummvm", "Demo")
            .with_runtime_hint("scummvm");
        let json = serde_json::to_value(content).expect("serialize descriptor");

        assert_eq!(json["schema"], CONTENT_DESCRIPTOR_SCHEMA);
        assert_eq!(json["locator"]["kind"], "FILE");
        assert_eq!(json["locator"]["path"], "games/demo.scummvm");
        assert_eq!(json["runtimeHint"], "scummvm");
        assert!(json["system"].is_null());
    }

    #[test]
    fn descriptor_rejects_empty_operator_fields() {
        assert!(ContentDescriptor::launch_target("", "Demo").validate().is_err());
        assert!(ContentDescriptor::file("", "Demo").validate().is_err());
        assert!(ContentDescriptor::directory("games/demo", "").validate().is_err());
        assert!(ContentDescriptor::file("games/demo", "Demo")
            .with_runtime_hint(" ")
            .validate()
            .is_err());
    }
}
