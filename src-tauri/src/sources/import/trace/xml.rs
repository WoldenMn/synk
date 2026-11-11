use std::io::BufRead;
use std::sync::atomic::{AtomicBool, Ordering};

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

use crate::error::AppError;
use crate::sources::import::limits::{too_big, Guarded, Limits};

const CANCEL_CHECK_EVERY: u32 = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    Open(String, Vec<(String, String)>),
    Close(String),
    Text(String),
}

fn invalid(what: &str, e: impl std::fmt::Display) -> AppError {
    AppError::Validation(format!("{what} illisible : {e}"))
}

fn local(e: &BytesStart<'_>) -> String {
    let name = e.local_name();
    let name: &str = name.as_ref();
    name.to_string()
}

fn open_of(e: &BytesStart<'_>, what: &str) -> Result<Tok, AppError> {
    let mut attrs = Vec::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|x| invalid(what, x))?;
        let value = attr
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|x| invalid(what, x))?
            .into_owned();
        let key: &str = attr.key.local_name().into_inner();
        attrs.push((key.to_string(), value));
    }
    Ok(Tok::Open(local(e), attrs))
}

pub fn attr<'a>(attrs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

fn check_root(
    tok: &Tok,
    depth: u32,
    seen_root: &mut bool,
    what: &str,
    root: &str,
) -> Result<(), AppError> {
    if depth != 0 || *seen_root {
        return Ok(());
    }
    *seen_root = true;
    match tok {
        Tok::Open(name, _) if name.eq_ignore_ascii_case(root) => Ok(()),
        _ => Err(AppError::Validation(format!(
            "{what} : ce fichier n'est pas un {root}"
        ))),
    }
}

pub fn scan<R, F>(
    source: R,
    what: &str,
    root: &str,
    cancel: &AtomicBool,
    on_token: F,
) -> Result<bool, AppError>
where
    R: BufRead,
    F: FnMut(Tok) -> Result<(), AppError>,
{
    scan_limited(source, &Limits::TRACE, what, root, cancel, on_token)
}

pub fn scan_limited<R, F>(
    source: R,
    limits: &Limits,
    what: &str,
    root: &str,
    cancel: &AtomicBool,
    mut on_token: F,
) -> Result<bool, AppError>
where
    R: BufRead,
    F: FnMut(Tok) -> Result<(), AppError>,
{
    let (guarded, gauge) = Guarded::new(source, limits.node);
    let mut reader = Reader::from_reader(guarded);
    let mut elements: u64 = 0;
    let mut buf = Vec::with_capacity(4096);
    let mut depth: u32 = 0;
    let mut seen_root = false;
    let mut ticks: u32 = 0;
    loop {
        ticks = ticks.wrapping_add(1);
        if ticks.is_multiple_of(CANCEL_CHECK_EVERY) && cancel.load(Ordering::Relaxed) {
            return Ok(true);
        }
        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| invalid(what, e))?;
        gauge.event_done();
        if matches!(event, Event::Start(_) | Event::Empty(_)) {
            elements += 1;
            if elements > limits.elements {
                return Err(too_big(what));
            }
        }
        match event {
            Event::Eof => break,
            Event::Start(e) => {
                if depth >= limits.depth {
                    return Err(invalid(what, "éléments imbriqués trop profondément"));
                }
                let tok = open_of(&e, what)?;
                check_root(&tok, depth, &mut seen_root, what, root)?;
                depth += 1;
                on_token(tok)?;
            }
            Event::Empty(e) => {
                let tok = open_of(&e, what)?;
                check_root(&tok, depth, &mut seen_root, what, root)?;
                on_token(tok)?;
                on_token(Tok::Close(local(&e)))?;
            }
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                let name = e.local_name();
                let name: &str = name.as_ref();
                on_token(Tok::Close(name.to_string()))?;
            }
            Event::Text(t) => {
                let text = t.xml10_content();
                if !text.trim().is_empty() {
                    on_token(Tok::Text(text.trim().to_string()))?;
                }
            }
            _ => {}
        }
        buf.clear();
    }
    if !seen_root {
        return Err(AppError::Validation(format!(
            "{what} : ce fichier n'est pas un {root}"
        )));
    }
    if depth != 0 {
        return Err(AppError::Validation(format!(
            "{what} tronqué : le fichier s'arrête avant la fin du document"
        )));
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn run(xml: &str, root: &str) -> Result<Vec<Tok>, AppError> {
        let mut out = Vec::new();
        scan(
            Cursor::new(xml.as_bytes()),
            "Fichier GPX",
            root,
            &AtomicBool::new(false),
            |t| {
                out.push(t);
                Ok(())
            },
        )?;
        Ok(out)
    }

    #[test]
    fn prefixes_are_dropped_and_empty_elements_open_and_close() {
        let toks = run(
            "<gpx a=\"1\"><gpxtpx:hr>140</gpxtpx:hr><trkpt lat=\"1\"/></gpx>",
            "gpx",
        )
        .unwrap();
        assert_eq!(
            toks[0],
            Tok::Open("gpx".into(), vec![("a".into(), "1".into())])
        );
        assert_eq!(toks[1], Tok::Open("hr".into(), vec![]));
        assert_eq!(toks[2], Tok::Text("140".into()));
        assert_eq!(toks[3], Tok::Close("hr".into()));
        assert_eq!(toks[5], Tok::Close("trkpt".into()));
    }

    #[test]
    fn a_truncated_document_and_a_foreign_root_are_named_errors() {
        let cut = run("<gpx><trk><trkpt lat=\"1\">", "gpx").unwrap_err();
        assert!(cut.to_string().contains("tronqué"), "{cut}");
        let foreign = run("<html><body/></html>", "gpx").unwrap_err();
        assert!(foreign.to_string().contains("pas un gpx"), "{foreign}");
        let empty = run("", "gpx").unwrap_err();
        assert!(empty.to_string().contains("pas un gpx"), "{empty}");
        let broken = run("<gpx><a></b></gpx>", "gpx").unwrap_err();
        assert!(broken.to_string().contains("illisible"), "{broken}");
    }

    fn run_limited(xml: &str, limits: &Limits) -> Result<Vec<Tok>, AppError> {
        let mut out = Vec::new();
        scan_limited(
            Cursor::new(xml.as_bytes()),
            limits,
            "Fichier GPX",
            "gpx",
            &AtomicBool::new(false),
            |t| {
                out.push(t);
                Ok(())
            },
        )?;
        Ok(out)
    }

    #[test]
    fn a_giant_text_node_a_deep_nesting_and_too_many_elements_are_refused() {
        let tight = Limits {
            bytes: 1024,
            node: 256,
            depth: 8,
            elements: 50,
        };
        let giant = format!("<gpx><name>{}</name></gpx>", "x".repeat(10_000));
        let err = run_limited(&giant, &tight).unwrap_err();
        assert!(err.to_string().contains("volumineux"), "{err}");

        let deep = format!("<gpx>{}{}</gpx>", "<a>".repeat(20), "</a>".repeat(20));
        let err = run_limited(&deep, &tight).unwrap_err();
        assert!(err.to_string().contains("profondément"), "{err}");

        let many = format!("<gpx>{}</gpx>", "<p/>".repeat(100));
        let err = run_limited(&many, &tight).unwrap_err();
        assert!(err.to_string().contains("limite"), "{err}");

        let fine = format!("<gpx>{}</gpx>", "<p/>".repeat(10));
        assert!(run_limited(&fine, &tight).is_ok());
    }
}
