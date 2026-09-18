use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

use crate::error::AppError;
use crate::settings::Settings;

const HTTP_TIMEOUT: StdDuration = StdDuration::from_secs(15);

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub trait NtfyClient {
    fn publish(&self, delay: DateTime<Utc>, kind: RappelKind) -> Result<String, AppError>;
}

pub struct ReqwestNtfy {
    settings: Settings,
    title: String,
    message: String,
    click: String,
    priority: u8,
}

impl ReqwestNtfy {
    pub fn for_rdv(
        settings: &Settings,
        client_nom: &str,
        debut: DateTime<Utc>,
        jitsi_url: &str,
        kind: RappelKind,
    ) -> Self {
        let local_debut = debut.with_timezone(&chrono::Local);
        let (title, message, priority) = match kind {
            RappelKind::H24 => (
                "Rappel RDV demain".to_string(),
                if client_nom.is_empty() {
                    format!("RDV le {}", local_debut.format("%d/%m %H:%M"))
                } else {
                    format!("RDV avec {} le {}", client_nom, local_debut.format("%d/%m %H:%M"))
                },
                4,
            ),
            RappelKind::H1 => (
                "Rappel RDV dans 1h".to_string(),
                if client_nom.is_empty() {
                    "RDV dans 1 heure".to_string()
                } else {
                    format!("RDV avec {} dans 1 heure", client_nom)
                },
                5,
            ),
        };
        Self {
            settings: settings.clone(),
            title,
            message,
            click: jitsi_url.to_string(),
            priority,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl ReqwestNtfy {
    pub async fn publish_async(&self, delay: DateTime<Utc>) -> Result<String, AppError> {
        ntfy_publish(
            &self.settings,
            &self.title,
            &self.message,
            &self.click,
            Some(delay),
            self.priority,
        )
        .await
    }

    pub fn publish_blocking(&self, delay: DateTime<Utc>) -> Result<String, AppError> {
        tauri::async_runtime::block_on(self.publish_async(delay))
    }
}

pub const NTFY_MAX: Duration = Duration::days(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RappelKind {
    H24,
    H1,
}

impl RappelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RappelKind::H24 => "24h",
            RappelKind::H1 => "1h",
        }
    }
}

pub fn echeance(debut: DateTime<Utc>, kind: RappelKind) -> DateTime<Utc> {
    match kind {
        RappelKind::H24 => debut - Duration::hours(24),
        RappelKind::H1 => debut - Duration::hours(1),
    }
}

pub fn dans_fenetre_ntfy(now: DateTime<Utc>, echeance: DateTime<Utc>) -> bool {
    echeance > now && echeance - now <= NTFY_MAX
}

pub fn should_publish(
    topic: &str,
    flag: bool,
    statut: &str,
    now: DateTime<Utc>,
    echeance: DateTime<Utc>,
    deja_programme: bool,
) -> bool {
    !topic.is_empty()
        && flag
        && statut == "planifie"
        && dans_fenetre_ntfy(now, echeance)
        && !deja_programme
}

#[derive(Serialize)]
struct NtfyPublishBody {
    topic: String,
    title: String,
    message: String,
    click: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    delay: Option<i64>,
    priority: u8,
}

pub async fn ntfy_publish(
    settings: &Settings,
    title: &str,
    message: &str,
    click: &str,
    delay: Option<DateTime<Utc>>,
    priority: u8,
) -> Result<String, AppError> {
    let ntfy = &settings.ntfy;
    let body = NtfyPublishBody {
        topic: ntfy.topic.clone(),
        title: title.to_string(),
        message: message.to_string(),
        click: click.to_string(),
        delay: delay.map(|d| d.timestamp()),
        priority,
    };

    let client = http_client();
    let mut req = client
        .post(&ntfy.serveur)
        .header("Content-Type", "application/json")
        .json(&body);

    if !ntfy.token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", ntfy.token));
    }

    let resp = req.send().await.map_err(|e| AppError::new(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::new(format!("ntfy publish: {} {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::new(e.to_string()))?;
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::new("ntfy: id manquant dans la reponse"))?;
    Ok(id.to_string())
}

pub async fn ntfy_delete(settings: &Settings, ntfy_id: &str) -> Result<(), AppError> {
    let ntfy = &settings.ntfy;
    let url = format!(
        "{}/{}/{}",
        ntfy.serveur.trim_end_matches('/'),
        ntfy.topic,
        ntfy_id
    );

    let client = http_client();
    let mut req = client.delete(&url);

    if !ntfy.token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", ntfy.token));
    }

    let resp = req.send().await.map_err(|e| AppError::new(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::new(format!("ntfy delete: {} {}", status, text)));
    }
    Ok(())
}

pub async fn ntfy_test(settings: &Settings) -> Result<(), AppError> {
    ntfy_publish(settings, "Synapt", "Synapt OK", "", None, 3).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn echeance_24h() {
        let d = chrono::DateTime::parse_from_rfc3339("2026-09-12T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            echeance(d, RappelKind::H24).to_rfc3339(),
            "2026-09-11T10:00:00+00:00"
        );
    }

    #[test]
    fn fenetre_4_jours_false() {
        let now = chrono::DateTime::parse_from_rfc3339("2026-09-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let e = now + chrono::Duration::days(4);
        assert!(!dans_fenetre_ntfy(now, e));
    }

    #[test]
    fn should_publish_sans_topic() {
        let now = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        let e = now + chrono::Duration::hours(1);
        assert!(!should_publish("", true, "planifie", now, e, false));
    }

    #[test]
    fn for_rdv_heure_locale() {
        let settings = Settings::default();
        let debut = chrono::DateTime::parse_from_rfc3339("2026-09-12T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let ntfy = ReqwestNtfy::for_rdv(&settings, "Martin", debut, "", RappelKind::H24);
        let local_debut = debut.with_timezone(&chrono::Local);
        let expected_msg = format!("RDV avec Martin le {}", local_debut.format("%d/%m %H:%M"));
        assert_eq!(ntfy.message(), expected_msg);

        let ntfy_sans_client = ReqwestNtfy::for_rdv(&settings, "", debut, "", RappelKind::H24);
        let expected_sans_client = format!("RDV le {}", local_debut.format("%d/%m %H:%M"));
        assert_eq!(ntfy_sans_client.message(), expected_sans_client);
    }
}
