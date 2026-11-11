use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use zip::ZipArchive;

use crate::error::AppError;
use crate::sources::import::limits::Limits;

const BUFFER_BYTES: usize = 256 * 1024;

pub struct Counting<R> {
    inner: R,
    seen: Arc<AtomicU64>,
    read: u64,
    limit: u64,
}

impl<R: Read> Read for Counting<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.seen.fetch_add(n as u64, Ordering::Relaxed);
        self.read += n as u64;
        if self.read > self.limit {
            return Err(std::io::Error::other(
                "volume lu au-delà de la limite acceptée, fichier refusé",
            ));
        }
        Ok(n)
    }
}

fn unreadable(what: &str, e: impl std::fmt::Display) -> AppError {
    AppError::Validation(format!("{what} : {e}"))
}

fn is_export_xml(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".xml") && !lower.contains("_cda") && lower.matches('/').count() <= 1
}

fn export_entry(archive: &mut ZipArchive<File>) -> Result<usize, AppError> {
    let mut best: Option<(usize, u64)> = None;
    for i in 0..archive.len() {
        let entry = archive
            .by_index_raw(i)
            .map_err(|e| unreadable("archive illisible", e))?;
        if entry.is_file() && entry.name().is_ok_and(|n| is_export_xml(&n)) {
            let size = entry.size();
            if best.is_none_or(|(_, s)| size > s) {
                best = Some((i, size));
            }
        }
    }
    best.map(|(i, _)| i).ok_or_else(|| {
        AppError::Validation(
            "l'archive ne contient pas de fichier export.xml : ce n'est pas un export Apple Santé"
                .into(),
        )
    })
}

pub fn with_export<T>(
    path: &Path,
    seen: Arc<AtomicU64>,
    f: impl FnOnce(&mut dyn BufRead, u64) -> Result<T, AppError>,
) -> Result<T, AppError> {
    with_export_limited(path, seen, &Limits::APPLE_HEALTH, f)
}

pub fn with_export_limited<T>(
    path: &Path,
    seen: Arc<AtomicU64>,
    limits: &Limits,
    f: impl FnOnce(&mut dyn BufRead, u64) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let file = File::open(path).map_err(|e| unreadable("fichier illisible", e))?;
    let is_zip = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("zip"));
    if !is_zip {
        let size = file
            .metadata()
            .map_err(|e| unreadable("fichier illisible", e))?
            .len();
        limits.check_size(size, "export Apple Santé")?;
        let counted = Counting {
            inner: file,
            seen,
            read: 0,
            limit: limits.bytes,
        };
        let mut reader = BufReader::with_capacity(BUFFER_BYTES, counted);
        return f(&mut reader, size);
    }
    let mut archive = ZipArchive::new(file).map_err(|e| unreadable("archive illisible", e))?;
    let index = export_entry(&mut archive)?;
    let entry = archive
        .by_index(index)
        .map_err(|e| unreadable("archive illisible", e))?;
    let size = entry.size();
    limits.check_size(size, "export Apple Santé")?;
    let counted = Counting {
        inner: entry,
        seen,
        read: 0,
        limit: limits.bytes,
    };
    let mut reader = BufReader::with_capacity(BUFFER_BYTES, counted);
    f(&mut reader, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_with(dir: &Path, entries: &[(&str, &[u8])]) -> std::path::PathBuf {
        let path = dir.join("export.zip");
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, data) in entries {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
        path
    }

    fn read_all(path: &Path) -> Result<(String, u64), AppError> {
        with_export(path, Arc::new(AtomicU64::new(0)), |r, size| {
            let mut text = String::new();
            r.read_to_string(&mut text).unwrap();
            Ok((text, size))
        })
    }

    #[test]
    fn the_export_xml_is_found_in_the_archive_and_the_cda_one_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = zip_with(
            dir.path(),
            &[
                (
                    "apple_health_export/export_cda.xml",
                    b"<cda>enorme enorme enorme</cda>",
                ),
                ("apple_health_export/export.xml", b"<HealthData/>"),
                ("apple_health_export/workout-routes/route_1.gpx", b"<gpx/>"),
            ],
        );
        let (text, size) = read_all(&path).unwrap();
        assert_eq!(text, "<HealthData/>");
        assert_eq!(size, 13);
    }

    #[test]
    fn a_localised_name_is_found_as_the_largest_xml() {
        let dir = tempfile::tempdir().unwrap();
        let path = zip_with(
            dir.path(),
            &[(
                "apple_health_export/exporter.xml",
                b"<HealthData locale=\"fr\"/>",
            )],
        );
        assert!(read_all(&path).unwrap().0.contains("fr"));
    }

    #[test]
    fn a_bare_xml_file_is_read_directly() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.xml");
        std::fs::write(&path, b"<HealthData/>").unwrap();
        assert_eq!(read_all(&path).unwrap().0, "<HealthData/>");
    }

    #[test]
    fn an_archive_without_an_export_and_a_broken_archive_are_clear_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = zip_with(dir.path(), &[("lisez-moi.txt", b"bonjour")]);
        let err = read_all(&path).unwrap_err();
        assert!(err.to_string().contains("export.xml"), "{err}");

        let broken = dir.path().join("casse.zip");
        std::fs::write(&broken, b"PK pas une archive").unwrap();
        assert!(read_all(&broken).is_err());
    }

    #[test]
    fn reading_past_the_limit_fails_whatever_the_declared_size_says() {
        let mut counted = Counting {
            inner: std::io::repeat(b'a'),
            seen: Arc::new(AtomicU64::new(0)),
            read: 0,
            limit: 1000,
        };
        let mut sink = Vec::new();
        let err = std::io::copy(&mut (&mut counted).take(5000), &mut sink).unwrap_err();
        assert!(err.to_string().contains("limite"), "{err}");
    }

    #[test]
    fn an_archive_whose_export_is_larger_than_the_limit_is_refused_before_reading() {
        let dir = tempfile::tempdir().unwrap();
        let path = zip_with(
            dir.path(),
            &[("apple_health_export/export.xml", &[b' '; 4096])],
        );
        let tight = Limits {
            bytes: 1024,
            ..Limits::APPLE_HEALTH
        };
        let err = with_export_limited(&path, Arc::new(AtomicU64::new(0)), &tight, |_, _| Ok(()))
            .unwrap_err();
        assert!(err.to_string().contains("limite"), "{err}");
    }
}
