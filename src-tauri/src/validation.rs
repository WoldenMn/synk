use crate::error::AppError;

pub fn check_f64_opt(label: &str, value: Option<f64>, min: f64, max: f64) -> Result<(), AppError> {
    if let Some(v) = value {
        if !v.is_finite() {
            return Err(AppError::Validation(format!(
                "{label} doit etre un nombre fini"
            )));
        }
        if v < min || v > max {
            return Err(AppError::Validation(format!(
                "{label} doit etre dans [{min}, {max}]"
            )));
        }
    }
    Ok(())
}

pub fn check_f64(label: &str, value: f64, min: f64, max: f64) -> Result<(), AppError> {
    check_f64_opt(label, Some(value), min, max)
}

pub fn check_i64_opt(label: &str, value: Option<i64>, min: i64, max: i64) -> Result<(), AppError> {
    if let Some(v) = value {
        if v < min || v > max {
            return Err(AppError::Validation(format!(
                "{label} doit etre dans [{min}, {max}]"
            )));
        }
    }
    Ok(())
}

pub fn check_text(label: &str, value: &str, max: usize) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{label} est requis")));
    }
    check_text_max(label, value, max)
}

pub fn check_text_max(label: &str, value: &str, max: usize) -> Result<(), AppError> {
    if value.chars().count() > max {
        return Err(AppError::Validation(format!(
            "{label} : {max} caracteres au plus"
        )));
    }
    Ok(())
}

pub fn check_text_opt(label: &str, value: Option<&str>, max: usize) -> Result<(), AppError> {
    value.map_or(Ok(()), |v| check_text_max(label, v, max))
}

pub fn check_one_of(label: &str, value: &str, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(AppError::Validation(format!(
        "{label} doit valoir : {}",
        allowed.join(", ")
    )))
}

pub fn check_date(label: &str, value: &str) -> Result<(), AppError> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| AppError::Validation(format!("{label} doit etre une date AAAA-MM-JJ")))
}

pub fn check_limit(limit: Option<i64>, default: i64) -> Result<i64, AppError> {
    let value = limit.unwrap_or(default);
    if !(1..=1000).contains(&value) {
        return Err(AppError::Validation(
            "Limite doit etre dans [1, 1000]".into(),
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_none_and_in_range() {
        assert!(check_f64_opt("x", None, 0.0, 100.0).is_ok());
        assert!(check_f64_opt("x", Some(50.0), 0.0, 100.0).is_ok());
        assert!(check_i64_opt("y", Some(10), 0, 100).is_ok());
    }

    #[test]
    fn rejects_nan_and_infinity() {
        assert!(check_f64_opt("x", Some(f64::NAN), 0.0, 100.0).is_err());
        assert!(check_f64_opt("x", Some(f64::INFINITY), 0.0, 100.0).is_err());
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(check_f64_opt("x", Some(-1.0), 0.0, 100.0).is_err());
        assert!(check_f64_opt("x", Some(101.0), 0.0, 100.0).is_err());
        assert!(check_i64_opt("y", Some(-1), 0, 100).is_err());
        assert!(check_i64_opt("y", Some(101), 0, 100).is_err());
    }
}
