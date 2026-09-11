use crate::error::AppError;

fn form_pct_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub async fn stripe_create_payment_link(
    secret: &str,
    nom: &str,
    prix_centimes: i64,
) -> Result<(String, String), AppError> {
    if prix_centimes == 0 {
        return Err(AppError::new("pas de lien"));
    }

    let body = format!(
        "line_items[0][quantity]=1&line_items[0][price_data][currency]=eur&line_items[0][price_data][unit_amount]={}&line_items[0][price_data][product_data][name]={}",
        prix_centimes,
        form_pct_encode(nom),
    );

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.stripe.com/v1/payment_links")
        .basic_auth(secret, Some(""))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::new(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::new(format!("stripe: {} {}", status, text)));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::new(e.to_string()))?;

    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::new("stripe: id manquant"))?;
    let url = json
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::new("stripe: url manquante"))?;

    Ok((id.to_string(), url.to_string()))
}
