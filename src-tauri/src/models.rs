use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub nom: String,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub statut: String,
    pub memo: Option<String>,
    pub tarif_id: Option<String>,
    pub date_naissance: Option<String>,
    pub urgence_nom: Option<String>,
    pub urgence_telephone: Option<String>,
    pub orientation: Option<String>,
    pub frequence: Option<String>,
    pub adresse: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct ClientWrite {
    pub id: Option<String>,
    pub nom: String,
    pub email: Option<String>,
    pub telephone: Option<String>,
    pub statut: Option<String>,
    pub memo: Option<String>,
    pub tarif_id: Option<String>,
    pub date_naissance: Option<String>,
    pub urgence_nom: Option<String>,
    pub urgence_telephone: Option<String>,
    pub orientation: Option<String>,
    pub frequence: Option<String>,
    pub adresse: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tarif {
    pub id: String,
    pub nom: String,
    pub duree_minutes: i64,
    pub prix_centimes: i64,
    pub prix_ttc: bool,
    pub actif: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rdv {
    pub id: String,
    pub client_id: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Honoraire {
    pub id: String,
    pub numero: String,
    pub client_id: String,
    pub client_nom: String,
    pub client_date_naissance: Option<String>,
    pub client_adresse: Option<String>,
    pub cabinet_nom: String,
    pub cabinet_adresse: Option<String>,
    pub cabinet_telephone: Option<String>,
    pub cabinet_email: Option<String>,
    pub cabinet_siret: Option<String>,
    pub mention_tva: String,
    pub moyen_paiement: String,
    pub statut: String,
    pub total_centimes: i64,
    pub annee: i32,
    pub seq: i32,
    pub pdf_relatif: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonoraireLigne {
    pub id: String,
    pub honoraire_id: String,
    pub rdv_id: String,
    pub debut: String,
    pub duree_minutes: i64,
    pub tarif_nom: String,
    pub prix_centimes: i64,
    pub actif: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonoraireDetail {
    pub honoraire: Honoraire,
    pub lignes: Vec<HonoraireLigne>,
}
