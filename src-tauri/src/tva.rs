//! TVA 20 % : un tarif est saisi HT ou TTC.

pub fn split_ttc(ttc_centimes: i64) -> (i64, i64) {
    if ttc_centimes <= 0 {
        return (0, 0);
    }
    let ht = (ttc_centimes * 100 + 60) / 120;
    (ht, ttc_centimes - ht)
}

pub fn ttc_from_ht(ht_centimes: i64) -> i64 {
    if ht_centimes <= 0 {
        return 0;
    }
    (ht_centimes * 120 + 50) / 100
}

pub fn montant_ttc(prix_centimes: i64, prix_ttc: bool) -> i64 {
    if prix_ttc {
        prix_centimes.max(0)
    } else {
        ttc_from_ht(prix_centimes)
    }
}

/// N° TVA intracommunautaire FR + clé + SIREN, à partir d'un SIRET à 14 chiffres.
pub fn tva_intra_from_siret(siret: &str) -> Option<String> {
    let digits: String = siret.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 14 {
        return None;
    }
    let siren: u64 = digits[..9].parse().ok()?;
    let key = (12 + 3 * (siren % 97)) % 97;
    Some(format!("FR{key:02}{}", &digits[..9]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_50_euros() {
        let (ht, tva) = split_ttc(5000);
        assert_eq!(ht + tva, 5000);
        assert_eq!(ht, 4167);
        assert_eq!(tva, 833);
    }

    #[test]
    fn split_zero() {
        assert_eq!(split_ttc(0), (0, 0));
    }

    #[test]
    fn ttc_from_ht_50() {
        assert_eq!(ttc_from_ht(5000), 6000);
        assert_eq!(montant_ttc(5000, false), 6000);
        assert_eq!(montant_ttc(5000, true), 5000);
        let (ht, tva) = split_ttc(ttc_from_ht(5000));
        assert_eq!(ht + tva, 6000);
        assert_eq!(ht, 5000);
    }

    #[test]
    fn tva_intra_siret_exemple() {
        // SIREN 404833048 → clé 04 (exemple INSEE courant)
        let n = tva_intra_from_siret("40483304800022").unwrap();
        assert!(n.starts_with("FR"));
        assert_eq!(n.len(), 13);
        assert_eq!(&n[4..], "404833048");
    }

    #[test]
    fn tva_intra_siret_invalide() {
        assert!(tva_intra_from_siret("123").is_none());
    }
}
