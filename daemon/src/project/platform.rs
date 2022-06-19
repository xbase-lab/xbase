use super::*;
use std::str::FromStr;
use xclog::XCBuildSettings;

#[derive(Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub enum Platform {
    #[serde(rename = "iOS")]
    IOS,
    #[serde(rename = "watchOS")]
    WatchOS,
    #[serde(rename = "tvOS")]
    TvOS,
    #[serde(rename = "macOS")]
    MacOS,
    Unknown,
}

impl Default for Platform {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Platform {
    pub fn from_sdk_root(sdk_root: &str) -> Self {
        match sdk_root {
            "iphoneos" => Self::IOS,
            "macosx" => Self::MacOS,
            "appletvos" => Self::TvOS,
            "watchos" => Self::WatchOS,
            _ => Self::Unknown,
        }
    }
    pub fn from_identifer(identifer: &str) -> Self {
        let name = identifer.replace("com.apple.CoreSimulator.SimRuntime.", "");
        let platform_str = name.split("-").next().unwrap().to_string();
        match Self::from_str(&platform_str) {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("Platfrom from str: {e}");
                Self::Unknown
            }
        }
    }

    pub fn get_from_settings(settings: &XCBuildSettings) -> Result<Platform> {
        let display = &settings.platform_display_name;
        let value = if display.contains("Simulator") {
            display
                .split(" ")
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .get(0)
                .ok_or_else(|| {
                    crate::Error::Message(format!("Unable to get Platfrom from `{display}`"))
                })?
                .to_string()
        } else {
            display.into()
        };
        Self::from_str(&value).map_err(|s| crate::Error::Message(s))
    }

    #[must_use]
    pub fn is_ios(&self) -> bool {
        matches!(self, Self::IOS)
    }

    #[must_use]
    pub fn is_watch_os(&self) -> bool {
        matches!(self, Self::WatchOS)
    }

    #[must_use]
    pub fn is_tv_os(&self) -> bool {
        matches!(self, Self::TvOS)
    }

    #[must_use]
    pub fn is_mac_os(&self) -> bool {
        matches!(self, Self::MacOS)
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, String> {
        match s {
            "iOS" => Ok(Platform::IOS),
            "watchOS" => Ok(Platform::WatchOS),
            "tvOS" => Ok(Platform::TvOS),
            "macOS" => Ok(Platform::MacOS),
            _ => Ok(Platform::Unknown),
        }
    }
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match self {
            Platform::IOS => "iOS",
            Platform::WatchOS => "watchOS",
            Platform::TvOS => "tvOS",
            Platform::MacOS => "macOS",
            _ => "",
        }
        .into()
    }
}
