use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub fn mention_tva_defaut() -> &'static str {
    "TVA au taux normal de 20 % (CGI, art. 278)"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CabinetSettings {
    #[serde(default)]
    pub nom: String,
    #[serde(default)]
    pub adresse: String,
    #[serde(default)]
    pub telephone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub siret: String,
    #[serde(default = "default_mention_tva")]
    pub mention_tva: String,
    #[serde(default)]
    pub prefixe_numero: String,
}

fn default_mention_tva() -> String {
    mention_tva_defaut().to_string()
}

impl Default for CabinetSettings {
    fn default() -> Self {
        Self {
            nom: String::new(),
            adresse: String::new(),
            telephone: String::new(),
            email: String::new(),
            siret: String::new(),
            mention_tva: default_mention_tva(),
            prefixe_numero: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub ntfy: NtfySettings,
    pub stripe: StripeSettings,
    #[serde(default)]
    pub cabinet: CabinetSettings,
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
    pub cabinet_nom: String,
    pub cabinet_adresse: String,
    pub cabinet_telephone: String,
    pub cabinet_email: String,
    pub cabinet_siret: String,
    pub mention_tva: String,
    pub prefixe_numero: String,
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
            cabinet: CabinetSettings::default(),
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
            cabinet_nom: self.cabinet.nom.clone(),
            cabinet_adresse: self.cabinet.adresse.clone(),
            cabinet_telephone: self.cabinet.telephone.clone(),
            cabinet_email: self.cabinet.email.clone(),
            cabinet_siret: self.cabinet.siret.clone(),
            mention_tva: self.cabinet.mention_tva.clone(),
            prefixe_numero: self.cabinet.prefixe_numero.clone(),
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
    let chars: Vec<char> = s.chars().collect();
    let last4: String = if chars.len() <= 4 {
        chars.into_iter().collect()
    } else {
        chars[chars.len() - 4..].iter().collect()
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

    #[test]
    fn mask_utf8_multibyte() {
        let (ok, last) = mask_secret("secret_sécurisé");
        assert!(ok);
        assert_eq!(last, "risé");
    }

    #[test]
    fn mask_short_or_empty() {
        let (ok, last) = mask_secret("abc");
        assert!(ok);
        assert_eq!(last, "abc");

        let (ok2, last2) = mask_secret("");
        assert!(!ok2);
        assert_eq!(last2, "");
    }
}
