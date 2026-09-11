use chrono::{DateTime, Duration, Utc};

use crate::error::AppError;

pub fn overlaps(
    a_start: &str,
    a_mins: i64,
    b_start: &str,
    b_mins: i64,
) -> Result<bool, AppError> {
    let a_start: DateTime<Utc> = a_start.parse()?;
    let b_start: DateTime<Utc> = b_start.parse()?;
    let a_end = a_start + Duration::minutes(a_mins);
    let b_end = b_start + Duration::minutes(b_mins);
    Ok(a_start < b_end && b_start < a_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chevauchement_simple() {
        assert!(overlaps(
            "2026-09-11T10:00:00Z",
            60,
            "2026-09-11T10:30:00Z",
            30
        )
        .unwrap());
    }

    #[test]
    fn adjacent_pas_chevauchement() {
        assert!(!overlaps(
            "2026-09-11T10:00:00Z",
            60,
            "2026-09-11T11:00:00Z",
            30
        )
        .unwrap());
    }
}
