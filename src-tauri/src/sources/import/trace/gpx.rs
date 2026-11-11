use std::io::BufRead;
use std::sync::atomic::AtomicBool;

use chrono::{DateTime, Utc};

use super::row::{workout_row, Draft};
use super::stats::{Point, TrackStats};
use super::xml::{attr, scan, Tok};
use super::Parsed;
use crate::error::AppError;
use crate::sources::import::rows::Skipped;

const WHAT: &str = "Fichier GPX";

pub fn time_of(text: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(text.trim())
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

#[derive(Default)]
struct State {
    path: Vec<String>,
    device: String,
    name: String,
    sport: String,
    stats: TrackStats,
    point: Option<Point>,
    in_track: bool,
    parsed: Parsed,
}

impl State {
    fn parent(&self) -> &str {
        let n = self.path.len();
        if n >= 2 {
            self.path[n - 2].as_str()
        } else {
            ""
        }
    }

    fn open(&mut self, name: String, attrs: &[(String, String)]) {
        match name.as_str() {
            "gpx" => self.device = attr(attrs, "creator").unwrap_or_default().to_string(),
            "trk" => {
                self.in_track = true;
                self.stats = TrackStats::default();
                self.name.clear();
                self.sport.clear();
            }
            "trkpt" if self.in_track => {
                self.point = Some(Point {
                    lat: attr(attrs, "lat").and_then(|v| v.parse().ok()),
                    lon: attr(attrs, "lon").and_then(|v| v.parse().ok()),
                    ..Point::default()
                });
            }
            _ => {}
        }
        self.path.push(name);
    }

    fn text(&mut self, text: &str) {
        let Some(current) = self.path.last().cloned() else {
            return;
        };
        let parent = self.parent().to_string();
        if let Some(point) = self.point.as_mut() {
            match current.as_str() {
                "time" if parent == "trkpt" => point.time = time_of(text),
                "ele" if parent == "trkpt" => point.elevation = text.parse().ok(),
                "hr" => point.heart_rate = text.parse().ok(),
                _ => {}
            }
        } else if self.in_track && parent == "trk" {
            match current.as_str() {
                "name" => self.name = text.to_string(),
                "type" => self.sport = text.to_string(),
                _ => {}
            }
        }
    }

    fn close(&mut self, name: &str) {
        self.path.pop();
        match name {
            "trkpt" => {
                if let Some(point) = self.point.take() {
                    self.stats.add(&point);
                }
            }
            "trk" => {
                self.in_track = false;
                self.finish_track();
            }
            _ => {}
        }
    }

    fn finish_track(&mut self) {
        let Some(summary) = self.stats.finish() else {
            self.parsed
                .skipped
                .add("trace : moins de deux points horodatés");
            return;
        };
        self.parsed.rows.push(workout_row(&Draft {
            prefix: "gpx",
            kind: "gpx",
            sport: &self.sport,
            device: &self.device,
            summary,
            duration: None,
            distance: Some(summary.distance),
            calories: None,
        }));
    }
}

pub fn parse<R: BufRead>(source: R, cancel: &AtomicBool) -> Result<Parsed, AppError> {
    let mut state = State::default();
    let cancelled = scan(source, WHAT, "gpx", cancel, |tok| {
        match tok {
            Tok::Open(name, attrs) => state.open(name, &attrs),
            Tok::Text(text) => state.text(&text),
            Tok::Close(name) => state.close(&name),
        }
        Ok(())
    })?;
    state.parsed.cancelled = cancelled;
    if state.parsed.rows.is_empty() && state.parsed.skipped == Skipped::default() && !cancelled {
        state
            .parsed
            .skipped
            .add("trace : aucune trace dans le fichier");
    }
    Ok(state.parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const FILE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Montre Test" xmlns="http://www.topografix.com/GPX/1/1" xmlns:gpxtpx="http://www.garmin.com/xmlschemas/TrackPointExtension/v1">
 <metadata><time>2026-10-25T09:59:00Z</time></metadata>
 <trk><name>Sortie du matin</name><type>running</type><trkseg>
  <trkpt lat="48.0000" lon="2.0000"><ele>100</ele><time>2026-10-25T10:00:00Z</time><extensions><gpxtpx:TrackPointExtension><gpxtpx:hr>120</gpxtpx:hr></gpxtpx:TrackPointExtension></extensions></trkpt>
  <trkpt lat="48.0010" lon="2.0000"><ele>112</ele><time>2026-10-25T10:05:00Z</time><extensions><gpxtpx:TrackPointExtension><gpxtpx:hr>150</gpxtpx:hr></gpxtpx:TrackPointExtension></extensions></trkpt>
  <trkpt lat="48.0020" lon="2.0000"><ele>108</ele><time>2026-10-25T10:10:00Z</time></trkpt>
 </trkseg></trk>
 <trk><name>Sans heures</name><trkseg><trkpt lat="48.0" lon="2.0"/><trkpt lat="48.1" lon="2.0"/></trkseg></trk>
</gpx>"#;

    fn parse_str(xml: impl AsRef<[u8]>) -> Result<Parsed, AppError> {
        parse(Cursor::new(xml.as_ref()), &AtomicBool::new(false))
    }

    #[test]
    fn a_track_becomes_a_workout_and_a_track_without_times_is_counted() {
        let parsed = parse_str(FILE).expect("gpx");
        assert_eq!(parsed.rows.len(), 1);
        let row = &parsed.rows[0];
        assert_eq!(row.category_label, "Course");
        assert_eq!(row.device, "Montre Test");
        assert_eq!(row.duration, 600);
        assert_eq!(row.start_at, "2026-10-25T10:00:00Z");
        assert_eq!(row.end_at, "2026-10-25T10:10:00Z");
        assert!((row.distance.unwrap() - 222.4).abs() < 1.0);
        assert_eq!(row.elevation, Some(12.0));
        assert_eq!(row.hr_average, Some(135));
        assert_eq!(row.hr_min, Some(120));
        assert_eq!(row.hr_max, Some(150));
        assert_eq!(parsed.skipped.total(), 1);
    }

    #[test]
    fn a_gpx_without_any_track_says_so() {
        let parsed = parse_str("<gpx creator=\"x\"><wpt lat=\"1\" lon=\"2\"/></gpx>").unwrap();
        assert!(parsed.rows.is_empty());
        assert_eq!(parsed.skipped.total(), 1);
    }

    #[test]
    fn a_truncated_file_writes_nothing_and_names_the_problem() {
        let clean_cut = &FILE.as_bytes()[..FILE.find("</trkseg>").unwrap()];
        let err = parse_str(clean_cut).unwrap_err();
        assert!(err.to_string().contains("tronqué"), "{err}");
        let err = parse_str(&FILE.as_bytes()[..FILE.len() / 2]).unwrap_err();
        assert!(err.to_string().contains("illisible"), "{err}");
        let err = parse_str("PK\u{3}\u{4} pas du xml").unwrap_err();
        assert!(err.to_string().contains("pas un gpx"), "{err}");
    }
}
