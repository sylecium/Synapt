use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub nom: String,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tarif {
    pub id: String,
    pub nom: String,
    pub duree_minutes: i64,
    pub prix_centimes: i64,
    pub actif: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rdv {
    pub id: String,
    pub client_id: String,
    pub tarif_id: Option<String>,
    pub debut: String,
    pub duree_minutes: i64,
    pub jitsi_url: String,
    pub stripe_url: Option<String>,
    pub stripe_id: Option<String>,
    pub note: Option<String>,
    pub statut: String,
    pub created_at: String,
    pub updated_at: String,
    pub client_nom: String,
    pub tarif_nom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub client_id: Option<String>,
    pub corps: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub aujourdhui: Vec<Rdv>,
    pub a_venir: Vec<Rdv>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RappelNtfy {
    pub id: String,
    pub rdv_id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub ntfy_id: Option<String>,
    pub echeance: String,
    pub etat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdvDetail {
    pub rdv: Rdv,
    pub rappels: Vec<RappelNtfy>,
}
