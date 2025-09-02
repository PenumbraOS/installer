use std::path::PathBuf;

pub struct Platform;

impl Platform {
    pub fn cache_dir() -> PathBuf {
        if let Some(cache_dir) = dirs::cache_dir() {
            cache_dir.join("penumbra-installer")
        } else {
            std::env::temp_dir().join("penumbra-installer")
        }
    }

    pub fn temp_dir() -> PathBuf {
        std::env::temp_dir().join("penumbra-installer")
    }

    pub fn executable_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            ".exe"
        } else {
            ""
        }
    }

    pub fn user_agent() -> String {
        format!(
            "PenumbraOS-Installer/{} ({})",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS
        )
    }
}
