use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

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

pub fn load_settings_from_path(path: &Path) -> Settings {
    let mut bak_path = path.to_path_buf();
    bak_path.as_mut_os_string().push(".bak");

    if let Ok(contents) = std::fs::read_to_string(path) {
        match serde_json::from_str::<Settings>(&contents) {
            Ok(s) => return s,
            Err(e) => {
                log::warn!(
                    "Fichier de réglages {:?} invalide ({}), tentative depuis {:?}",
                    path,
                    e,
                    bak_path
                );
            }
        }
    }

    if let Ok(bak_contents) = std::fs::read_to_string(&bak_path) {
        match serde_json::from_str::<Settings>(&bak_contents) {
            Ok(s) => {
                log::warn!(
                    "Réglages restaurés avec succès depuis la copie de sauvegarde {:?}",
                    bak_path
                );
                return s;
            }
            Err(e) => {
                log::error!(
                    "Échec de lecture de la copie de sauvegarde {:?}: {}",
                    bak_path,
                    e
                );
            }
        }
    }

    Settings::default()
}

pub fn save_settings_to_path(s: &Settings, path: &Path) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(s).map_err(|e| AppError::new(e.to_string()))?;
    let mut tmp_path = path.to_path_buf();
    tmp_path.as_mut_os_string().push(".tmp");
    let mut bak_path = path.to_path_buf();
    bak_path.as_mut_os_string().push(".bak");

    let write_res = (|| -> Result<(), std::io::Error> {
        #[cfg(unix)]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt;
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&tmp_path)?
        };
        #[cfg(not(unix))]
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;

        file.write_all(json.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::new(format!("Échec d'écriture des réglages: {}", e)));
    }

    if path.exists() {
        let _ = std::fs::copy(path, &bak_path);
    }

    if let Err(e) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(AppError::new(format!(
            "Échec de renommage atomique des réglages: {}",
            e
        )));
    }

    Ok(())
}

pub fn load_settings() -> Settings {
    load_settings_from_path(&settings_path())
}

pub fn save_settings(s: &Settings) -> Result<(), AppError> {
    save_settings_to_path(s, &settings_path())
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

    #[test]
    fn save_and_load_settings_atomique_et_backup() {
        let temp_dir =
            std::env::temp_dir().join(format!("synapt_test_settings_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("settings.json");

        let mut s1 = Settings::default();
        s1.cabinet.nom = "Dr. Test".to_string();
        save_settings_to_path(&s1, &path).expect("save 1 ok");

        assert!(path.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        let loaded1 = load_settings_from_path(&path);
        assert_eq!(loaded1.cabinet.nom, "Dr. Test");

        let mut s2 = s1.clone();
        s2.cabinet.nom = "Dr. Modifié".to_string();
        save_settings_to_path(&s2, &path).expect("save 2 ok");

        let mut bak_path = path.clone();
        bak_path.as_mut_os_string().push(".bak");
        assert!(bak_path.exists());

        let bak_content = std::fs::read_to_string(&bak_path).expect("read bak");
        let bak_settings: Settings = serde_json::from_str(&bak_content).expect("parse bak");
        assert_eq!(bak_settings.cabinet.nom, "Dr. Test");

        let loaded2 = load_settings_from_path(&path);
        assert_eq!(loaded2.cabinet.nom, "Dr. Modifié");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn load_settings_fallback_sur_corruption() {
        let temp_dir =
            std::env::temp_dir().join(format!("synapt_test_settings_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let path = temp_dir.join("settings.json");

        let mut s1 = Settings::default();
        s1.cabinet.nom = "Version Secours".to_string();
        save_settings_to_path(&s1, &path).unwrap();

        let mut s2 = s1.clone();
        s2.cabinet.nom = "Version Courante".to_string();
        save_settings_to_path(&s2, &path).unwrap();

        std::fs::write(&path, "{ corrupt json ...").unwrap();

        let recovered = load_settings_from_path(&path);
        assert_eq!(recovered.cabinet.nom, "Version Secours");

        let mut bak_path = path.clone();
        bak_path.as_mut_os_string().push(".bak");
        std::fs::write(&bak_path, "not json either").unwrap();

        let fallback = load_settings_from_path(&path);
        assert_eq!(fallback.cabinet.nom, "");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
