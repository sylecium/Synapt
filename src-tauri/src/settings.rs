use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub ntfy: NtfySettings,
    pub stripe: StripeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NtfySettings {
    pub serveur: String,
    pub topic: String,
    pub token: String,
    pub rappel_24h: bool,
    pub rappel_1h: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StripeSettings {
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SettingsPublic {
    pub ntfy_serveur: String,
    pub ntfy_topic: String,
    pub ntfy_token_configured: bool,
    pub ntfy_token_last4: String,
    pub rappel_24h: bool,
    pub rappel_1h: bool,
    pub stripe_configured: bool,
    pub stripe_last4: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ntfy: NtfySettings {
                serveur: "https://ntfy.sh".to_string(),
                topic: String::new(),
                token: String::new(),
                rappel_24h: true,
                rappel_1h: true,
            },
            stripe: StripeSettings {
                secret_key: String::new(),
            },
        }
    }
}

impl Settings {
    pub fn to_public(&self) -> SettingsPublic {
        let (ntfy_token_configured, ntfy_token_last4) = mask_secret(&self.ntfy.token);
        let (stripe_configured, stripe_last4) = mask_secret(&self.stripe.secret_key);
        SettingsPublic {
            ntfy_serveur: self.ntfy.serveur.clone(),
            ntfy_topic: self.ntfy.topic.clone(),
            ntfy_token_configured,
            ntfy_token_last4,
            rappel_24h: self.ntfy.rappel_24h,
            rappel_1h: self.ntfy.rappel_1h,
            stripe_configured,
            stripe_last4,
        }
    }
}

pub fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("synapt")
        .join("settings.json")
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

pub fn save_settings(s: &Settings) -> Result<(), AppError> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(s).map_err(|e| AppError::new(e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        file.write_all(json.as_bytes())?;
    }
    #[cfg(not(unix))]
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)?;
        file.write_all(json.as_bytes())?;
    }
    Ok(())
}

pub fn mask_secret(s: &str) -> (bool, String) {
    if s.is_empty() {
        return (false, String::new());
    }
    let last4 = if s.len() <= 4 {
        s.to_string()
    } else {
        s[s.len() - 4..].to_string()
    };
    (true, last4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_last4() {
        let (ok, last) = mask_secret("sk_test_abcdefghij");
        assert!(ok);
        assert_eq!(last, "ghij");
    }
}
