use std::fs;
use std::path::Path;

use chrono::Local;
use printpdf::*;

use crate::error::AppError;
use crate::models::{HonoraireDetail, HonoraireLigne};
use crate::repo;
use crate::tva::{split_ttc, tva_intra_from_siret};

const DEJA_VU_SANS: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
const DEJA_VU_SANS_BOLD: &[u8] = include_bytes!("../fonts/DejaVuSans-Bold.ttf");

const PAGE_TOP: f32 = 277.0;
const PAGE_BOTTOM: f32 = 22.0;
const LINE: f32 = 4.6;
const LEFT: f32 = 18.0;
const RIGHT_COL: f32 = 112.0;
const COL_RIGHT: f32 = 192.0;
const COL_DATE: f32 = 88.0;
const COL_QTE: f32 = 114.0;
const COL_HT: f32 = 128.0;
const COL_TVA: f32 = 152.0;
const COL_TTC: f32 = 172.0;
const TOTALS_BLOCK_HEIGHT: f32 = 65.0;

pub fn write_pdf(detail: &HonoraireDetail, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = render_pdf(detail)?;
    fs::write(dest, bytes)?;
    Ok(())
}

struct PdfBuilder {
    pages: Vec<PdfPage>,
    ops: Vec<Op>,
    y: f32,
    regular: PdfFontHandle,
    bold: PdfFontHandle,
    annulee: bool,
}

impl PdfBuilder {
    fn new(regular: PdfFontHandle, bold: PdfFontHandle, annulee: bool) -> Self {
        Self {
            pages: Vec::new(),
            ops: Vec::new(),
            y: PAGE_TOP,
            regular,
            bold,
            annulee,
        }
    }

    fn text_at(&mut self, x: f32, y: f32, font: &PdfFontHandle, size: f32, text: &str) {
        if text.is_empty() {
            return;
        }
        self.ops.push(Op::StartTextSection);
        self.ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(x).into(),
                y: Mm(y).into(),
            },
        });
        self.ops.push(Op::SetFont {
            font: font.clone(),
            size: Pt(size),
        });
        self.ops.push(Op::ShowText {
            items: vec![TextItem::Text(text.to_string())],
        });
        self.ops.push(Op::EndTextSection);
    }

    fn emit(&mut self, font: &PdfFontHandle, size: f32, text: &str) {
        let t = text.trim();
        if t.is_empty() {
            return;
        }
        self.ensure_space();
        self.text_at(LEFT, self.y, font, size, t);
        self.y -= LINE;
    }

    fn emit_x(&mut self, x: f32, font: &PdfFontHandle, size: f32, text: &str) {
        let t = text.trim();
        if t.is_empty() {
            return;
        }
        self.text_at(x, self.y, font, size, t);
    }

    fn hline(&mut self, y: f32) {
        self.ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb::new(0.25, 0.25, 0.25, None)),
        });
        self.ops.push(Op::SetOutlineThickness { pt: Pt(0.35) });
        self.ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(LEFT), Mm(y)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(COL_RIGHT), Mm(y)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
    }

    fn ensure_space(&mut self) {
        if self.y < PAGE_BOTTOM {
            self.flush_page();
            self.y = PAGE_TOP;
        }
    }

    fn gap(&mut self, mm: f32) {
        self.y -= mm;
    }

    fn flush_page(&mut self) {
        if self.ops.is_empty() {
            return;
        }
        if self.annulee {
            self.text_at(70.0, 150.0, &self.bold.clone(), 32.0, "ANNULÉE");
        }
        self.pages
            .push(PdfPage::new(Mm(210.0), Mm(297.0), self.ops.clone()));
        self.ops.clear();
    }

    fn finish(mut self) -> Vec<PdfPage> {
        self.flush_page();
        self.pages
    }
}

fn render_pdf(detail: &HonoraireDetail) -> Result<Vec<u8>, AppError> {
    let mut font_warnings = Vec::new();
    let font = ParsedFont::from_bytes(DEJA_VU_SANS, 0, &mut font_warnings)
        .ok_or_else(|| AppError::new("Impossible d'enregistrer le PDF dans le dossier Synapt."))?;
    let font_bold = ParsedFont::from_bytes(DEJA_VU_SANS_BOLD, 0, &mut font_warnings)
        .ok_or_else(|| AppError::new("Impossible d'enregistrer le PDF dans le dossier Synapt."))?;

    let mut doc = PdfDocument::new("Facture");
    let regular = PdfFontHandle::External(doc.add_font(&font));
    let bold = PdfFontHandle::External(doc.add_font(&font_bold));

    let h = &detail.honoraire;
    let mut b = PdfBuilder::new(regular.clone(), bold.clone(), h.statut == "annulee");
    let date_emission = format_created_at(&h.created_at);

    b.text_at(LEFT, b.y, &bold, 18.0, "FACTURE");
    b.text_at(128.0, b.y, &regular, 10.0, &format!("n° {}", h.numero));
    b.y -= 6.0;
    b.text_at(LEFT, b.y, &bold, 10.0, "Facture acquittée");
    b.text_at(
        128.0,
        b.y,
        &regular,
        10.0,
        &format!("Date d'émission : {date_emission}"),
    );
    b.y -= LINE;
    b.emit(&regular, 9.0, "Note d'honoraires - prestation de services");
    b.gap(3.0);
    b.hline(b.y + 2.0);
    b.gap(4.0);

    b.emit_x(LEFT, &bold, 10.0, "Prestataire");
    b.emit_x(RIGHT_COL, &bold, 10.0, "Client");
    b.y -= LINE;

    let mut y_left = b.y;
    let mut y_right = b.y;

    fn col(b: &mut PdfBuilder, x: f32, y: &mut f32, font: &PdfFontHandle, size: f32, text: &str) {
        let t = text.trim();
        if t.is_empty() {
            return;
        }
        b.text_at(x, *y, font, size, t);
        *y -= LINE;
    }

    col(&mut b, LEFT, &mut y_left, &bold, 10.0, &h.cabinet_nom);
    if let Some(ref a) = h.cabinet_adresse {
        for line in wrap(a, 42) {
            col(&mut b, LEFT, &mut y_left, &regular, 9.0, &line);
        }
    }
    if let Some(ref t) = h.cabinet_telephone {
        col(
            &mut b,
            LEFT,
            &mut y_left,
            &regular,
            9.0,
            &format!("Tél. {t}"),
        );
    }
    if let Some(ref e) = h.cabinet_email {
        col(&mut b, LEFT, &mut y_left, &regular, 9.0, e);
    }
    if let Some(ref s) = h.cabinet_siret {
        col(
            &mut b,
            LEFT,
            &mut y_left,
            &regular,
            9.0,
            &format!("SIRET {s}"),
        );
        if let Some(tva) = tva_intra_from_siret(s) {
            col(
                &mut b,
                LEFT,
                &mut y_left,
                &regular,
                9.0,
                &format!("N° TVA intracommunautaire {tva}"),
            );
        }
    }

    col(&mut b, RIGHT_COL, &mut y_right, &bold, 10.0, &h.client_nom);
    if let Some(ref a) = h.client_adresse {
        for line in wrap(a, 38) {
            col(&mut b, RIGHT_COL, &mut y_right, &regular, 9.0, &line);
        }
    }
    if let Some(ref d) = h.client_date_naissance {
        if let Some(fmt) = format_date_naissance(d) {
            col(
                &mut b,
                RIGHT_COL,
                &mut y_right,
                &regular,
                9.0,
                &format!("Né(e) le {fmt}"),
            );
        }
    }

    b.y = y_left.min(y_right) - 5.0;

    table_header(&mut b);
    let mut total_ht = 0_i64;
    let mut total_tva = 0_i64;
    for ligne in &detail.lignes {
        if b.y < PAGE_BOTTOM + LINE * 3.0 {
            b.flush_page();
            b.y = PAGE_TOP;
            table_header(&mut b);
        }
        let (ht, tva) = split_ttc(ligne.prix_centimes);
        total_ht += ht;
        total_tva += tva;
        table_row(&mut b, ligne, ht, tva);
    }
    b.hline(b.y + 2.4);
    if b.y < PAGE_BOTTOM + TOTALS_BLOCK_HEIGHT {
        b.flush_page();
        b.y = PAGE_TOP;
    } else {
        b.gap(4.0);
    }

    b.emit_x(128.0, &regular, 10.0, "Total HT");
    b.emit_x(168.0, &regular, 10.0, &format_centimes(total_ht));
    b.y -= LINE;
    b.emit_x(128.0, &regular, 10.0, "TVA 20 %");
    b.emit_x(168.0, &regular, 10.0, &format_centimes(total_tva));
    b.y -= LINE;
    b.emit_x(128.0, &bold, 11.0, "Total TTC");
    b.emit_x(168.0, &bold, 11.0, &format_centimes(h.total_centimes));
    b.y -= LINE + 4.0;

    b.emit(
        &regular,
        9.0,
        &format!(
            "Arrêtée la présente facture à la somme de {}.",
            montant_en_lettres(h.total_centimes)
        ),
    );
    b.gap(2.0);
    b.emit(
        &bold,
        11.0,
        &format!(
            "Acquittée le {date_emission} - {}",
            moyen_label(&h.moyen_paiement)
        ),
    );
    b.gap(4.0);
    b.emit(
        &regular,
        8.0,
        "TVA au taux normal de 20 % (CGI, art. 278). TVA due d'après les encaissements (CGI, art. 269).",
    );
    b.emit(
        &regular,
        8.0,
        "Pas d'escompte pour paiement anticipé. Facture acquittée, sans pénalité de retard.",
    );
    if let Some(extra) = extra_mention(&h.mention_tva) {
        b.emit(&regular, 8.0, extra);
    }
    b.gap(8.0);
    b.emit(&regular, 10.0, "Pour acquit");
    b.y -= 12.0;

    let pages = b.finish();
    let mut warnings = Vec::new();
    Ok(doc.with_pages(pages).save(
        &PdfSaveOptions {
            subset_fonts: true,
            ..Default::default()
        },
        &mut warnings,
    ))
}

fn extra_mention(raw: &str) -> Option<&str> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    if t == crate::settings::mention_tva_defaut() {
        return None;
    }
    if t.to_lowercase().contains("non applicable") {
        return None;
    }
    Some(t)
}

fn wrap(s: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in s.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut cur = String::new();
        let mut cur_len = 0;
        for word in trimmed.split_whitespace() {
            let w_len = word.chars().count();
            if !cur.is_empty() && cur_len + 1 + w_len > max {
                lines.push(cur);
                cur = String::new();
                cur_len = 0;
            }
            if !cur.is_empty() {
                cur.push(' ');
                cur_len += 1;
            }
            cur.push_str(word);
            cur_len += w_len;
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
    }
    if lines.is_empty() && !s.trim().is_empty() {
        lines.push(s.trim().to_string());
    }
    lines
}

fn table_header(b: &mut PdfBuilder) {
    b.ensure_space();
    b.hline(b.y + 2.4);
    b.y -= 1.2;
    let cols = [
        (LEFT, "Désignation"),
        (COL_DATE, "Date"),
        (COL_QTE, "Qté"),
        (COL_HT, "PU HT"),
        (COL_TVA, "TVA 20 %"),
        (COL_TTC, "TTC"),
    ];
    for (x, label) in cols {
        b.text_at(x, b.y, &b.bold.clone(), 8.5, label);
    }
    b.y -= LINE;
    b.hline(b.y + 2.4);
    b.y -= 1.0;
}

fn table_row(b: &mut PdfBuilder, ligne: &HonoraireLigne, ht: i64, tva: i64) {
    let local = repo::parse_debut_utc(&ligne.debut)
        .ok()
        .map(|utc| utc.with_timezone(&Local));
    let date_s = local
        .map(|dt| dt.format("%d/%m/%Y").to_string())
        .unwrap_or_default();
    let designation = {
        let heure = local
            .map(|dt| dt.format("%H:%M").to_string())
            .unwrap_or_default();
        if heure.is_empty() {
            format!("1 séance - {}", ligne.tarif_nom)
        } else {
            format!("1 séance - {} ({heure})", ligne.tarif_nom)
        }
    };
    let cols = [
        (LEFT, truncate(&designation, 32)),
        (COL_DATE, date_s),
        (COL_QTE, "1".to_string()),
        (COL_HT, format_centimes(ht)),
        (COL_TVA, format_centimes(tva)),
        (COL_TTC, format_centimes(ligne.prix_centimes)),
    ];
    for (x, text) in cols {
        b.text_at(x, b.y, &b.regular.clone(), 8.5, &text);
    }
    b.y -= LINE;
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}...")
    }
}

fn format_created_at(iso: &str) -> String {
    repo::parse_debut_utc(iso)
        .ok()
        .map(|utc| utc.with_timezone(&Local).format("%d/%m/%Y").to_string())
        .unwrap_or_else(|| iso.to_string())
}

fn format_date_naissance(raw: &str) -> Option<String> {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d") {
        return Some(d.format("%d/%m/%Y").to_string());
    }
    let t = raw.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn format_centimes(centimes: i64) -> String {
    let euros = centimes / 100;
    let cents = centimes.rem_euclid(100);
    format!("{},{:02} €", euros, cents)
}

fn moyen_label(m: &str) -> &str {
    match m {
        "especes" => "Espèces",
        "cheque" => "Chèque",
        "cb" => "Carte",
        "stripe" => "Stripe",
        _ => m,
    }
}

fn montant_en_lettres(centimes: i64) -> String {
    let n = centimes.max(0) as u64;
    let euros = n / 100;
    let cents = n % 100;
    let e = match euros {
        0 => "zéro euro".to_string(),
        1 => "un euro".to_string(),
        _ => format!("{} euros", nombre_fr(euros)),
    };
    match cents {
        0 => e,
        1 => format!("{e} et un centime"),
        _ => format!("{e} et {} centimes", nombre_fr(cents)),
    }
}

fn nombre_fr(n: u64) -> String {
    if n == 0 {
        return "zéro".into();
    }
    let milliers = n / 1000;
    let reste = n % 1000;
    let mut s = String::new();
    if milliers == 1 {
        s.push_str("mille");
    } else if milliers > 0 {
        s.push_str(&below_thousand(milliers));
        s.push_str(" mille");
    }
    let rest = below_thousand(reste);
    if !rest.is_empty() {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&rest);
    }
    s
}

fn below_thousand(n: u64) -> String {
    let h = n / 100;
    let r = n % 100;
    let mut s = String::new();
    if h == 1 {
        s.push_str("cent");
    } else if h > 1 {
        s.push_str(UNITS[h as usize]);
        s.push_str(if r == 0 { " cents" } else { " cent" });
    }
    let rest = below_hundred(r);
    if !rest.is_empty() {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(&rest);
    }
    s
}

const UNITS: [&str; 17] = [
    "", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf", "dix", "onze",
    "douze", "treize", "quatorze", "quinze", "seize",
];

fn below_hundred(n: u64) -> String {
    match n {
        0 => String::new(),
        1..=16 => UNITS[n as usize].to_string(),
        17..=19 => format!("dix-{}", UNITS[(n - 10) as usize]),
        20 => "vingt".into(),
        21 => "vingt-et-un".into(),
        30 => "trente".into(),
        31 => "trente-et-un".into(),
        40 => "quarante".into(),
        41 => "quarante-et-un".into(),
        50 => "cinquante".into(),
        51 => "cinquante-et-un".into(),
        60 => "soixante".into(),
        61 => "soixante-et-un".into(),
        71 => "soixante-et-onze".into(),
        80 => "quatre-vingts".into(),
        81 => "quatre-vingt-un".into(),
        91 => "quatre-vingt-onze".into(),
        22..=29 => format!("vingt-{}", UNITS[(n % 10) as usize]),
        32..=39 => format!("trente-{}", UNITS[(n % 10) as usize]),
        42..=49 => format!("quarante-{}", UNITS[(n % 10) as usize]),
        52..=59 => format!("cinquante-{}", UNITS[(n % 10) as usize]),
        62..=69 => format!("soixante-{}", UNITS[(n % 10) as usize]),
        70..=79 => format!("soixante-{}", below_hundred(n - 60)),
        82..=99 => format!("quatre-vingt-{}", below_hundred(n - 80)),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lettres_50_euros() {
        assert_eq!(montant_en_lettres(5000), "cinquante euros");
    }

    #[test]
    fn lettres_41_67() {
        assert_eq!(
            montant_en_lettres(4167),
            "quarante-et-un euros et soixante-sept centimes"
        );
    }

    #[test]
    fn lettres_zero() {
        assert_eq!(montant_en_lettres(0), "zéro euro");
    }

    #[test]
    fn lettres_80() {
        assert_eq!(montant_en_lettres(8000), "quatre-vingts euros");
    }

    #[test]
    fn lettres_un_euro_un_centime() {
        assert_eq!(montant_en_lettres(101), "un euro et un centime");
    }

    #[test]
    fn wrap_multiligne_preserve_les_lignes() {
        let input = "12 rue des Lilas\nBâtiment B\n75011 Paris";
        let wrapped = wrap(input, 40);
        assert_eq!(
            wrapped,
            vec!["12 rue des Lilas", "Bâtiment B", "75011 Paris"]
        );
    }

    #[test]
    fn wrap_ligne_longue_decoupe_mots() {
        let input = "Une première ligne très longue qui dépasse le maximum autorisé\nDeuxième ligne courte";
        let wrapped = wrap(input, 25);
        assert_eq!(wrapped[0], "Une première ligne très");
        assert_eq!(wrapped[1], "longue qui dépasse le");
        assert_eq!(wrapped[2], "maximum autorisé");
        assert_eq!(wrapped[3], "Deuxième ligne courte");
    }

    #[test]
    fn wrap_ignore_lignes_vides() {
        let input = "Première ligne\n\n   \nDeuxième ligne";
        let wrapped = wrap(input, 40);
        assert_eq!(wrapped, vec!["Première ligne", "Deuxième ligne"]);
    }
}
