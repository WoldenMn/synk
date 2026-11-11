use std::panic::{catch_unwind, AssertUnwindSafe};

use fitparser::{ErrorKind, FitDataRecord};

use crate::error::AppError;

const MAX_BYTES: u64 = 256 * 1024 * 1024;

fn named(kind: &ErrorKind) -> AppError {
    let text = match kind {
        ErrorKind::InvalidCrc(..) => {
            "fichier FIT corrompu : la somme de contrôle ne correspond pas".to_string()
        }
        ErrorKind::UnexpectedEof(_) => "fichier FIT tronqué : il s'arrête trop tôt".to_string(),
        ErrorKind::TrailingBytes(n) => {
            format!("fichier FIT illisible : {n} octets en trop après la fin")
        }
        ErrorKind::ParseError(position, _) => {
            format!("fichier FIT illisible : structure invalide à l'octet {position}")
        }
        ErrorKind::MissingDefinitionMessage(..)
        | ErrorKind::MissingDeveloperDefinitionMessage() => {
            "fichier FIT illisible : un message n'a pas de définition".to_string()
        }
        ErrorKind::Io(e) => format!("fichier FIT illisible : {e}"),
        ErrorKind::ValueError(e) => format!("fichier FIT illisible : {e}"),
    };
    AppError::Validation(text)
}

pub fn check_size(bytes: u64) -> Result<(), AppError> {
    if bytes > MAX_BYTES {
        return Err(AppError::Validation(
            "fichier FIT trop volumineux (plus de 256 Mo)".into(),
        ));
    }
    Ok(())
}

fn truncated() -> AppError {
    AppError::Validation(
        "fichier FIT tronqué : il est plus court que son en-tête ne l'annonce".into(),
    )
}

pub fn check_frame(bytes: &[u8]) -> Result<(), AppError> {
    let mut offset = 0usize;
    if bytes.is_empty() {
        return Err(AppError::Validation("fichier FIT vide".into()));
    }
    while offset < bytes.len() {
        let rest = &bytes[offset..];
        let header_size = usize::from(rest[0]);
        if rest.len() < 12 || (header_size != 12 && header_size != 14) || &rest[8..12] != b".FIT" {
            return Err(AppError::Validation(
                "fichier FIT illisible : l'en-tête .FIT est absent".into(),
            ));
        }
        let data_size = u32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]) as usize;
        let total = header_size + data_size + 2;
        if rest.len() < total {
            return Err(truncated());
        }
        offset += total;
    }
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<Vec<FitDataRecord>, AppError> {
    check_frame(bytes)?;
    let outcome = catch_unwind(AssertUnwindSafe(|| fitparser::from_bytes(bytes)));
    match outcome {
        Ok(Ok(records)) => Ok(records),
        Ok(Err(e)) => Err(named(&e)),
        Err(_) => Err(AppError::Validation(
            "fichier FIT illisible : la lecture a échoué sur une structure incohérente".into(),
        )),
    }
}
