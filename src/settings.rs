// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::display::{Precision, Scale};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Place {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub scale: Scale,
    pub precision: Precision,
    pub place: Option<Place>,
    pub tour_seen: bool,
    pub sidebar: bool,
    pub timer_spans: f64,
    pub timer_seconds: f64,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            scale: Scale::Spans,
            precision: Precision::Medium,
            place: None,
            tour_seen: false,
            sidebar: true,
            timer_spans: 5.0,
            timer_seconds: 1000.0,
        }
    }
}

impl Settings {
    pub fn load() -> Settings {
        load("settings.json").unwrap_or_default()
    }

    pub fn save(&self) {
        save("settings.json", self);
    }
}

#[cfg(target_os = "linux")]
fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_default()
        .join("bogentag")
}

#[cfg(target_os = "macos")]
fn config_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("Library/Application Support/Bogentag")
}

#[cfg(windows)]
fn config_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join("Bogentag")
}

pub fn load<T: DeserializeOwned>(name: &str) -> Option<T> {
    fs::read_to_string(config_dir().join(name))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
}

/// Written beside the file and renamed over it, readable by the owner only,
/// so a crash mid-write never leaves half a file.
pub fn save<T: Serialize>(name: &str, value: &T) {
    let dir = config_dir();
    if fs::create_dir_all(&dir).is_ok()
        && let Ok(text) = serde_json::to_string_pretty(value)
    {
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let path = dir.join(name);
        let partial = path.with_extension("partial");
        if let Ok(mut file) = options.open(&partial)
            && file.write_all(text.as_bytes()).is_ok()
        {
            let _ = fs::rename(&partial, &path);
        }
    }
}
