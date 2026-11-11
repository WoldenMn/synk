use std::io::BufRead;
use std::sync::atomic::AtomicBool;

use chrono::Duration;

use super::gpx::time_of;
use super::row::{workout_row, Draft};
use super::stats::{Point, Summary, TrackStats};
use super::xml::{attr, scan, Tok};
use super::Parsed;
use crate::error::AppError;
use crate::sources::import::rows::Skipped;

const WHAT: &str = "Fichier TCX";

#[derive(Default)]
struct Laps {
    seconds: f64,
    distance: f64,
    calories: f64,
    hr_average_sum: f64,
    hr_average_count: u32,
    hr_max: Option<f64>,
}

#[derive(Default)]
struct State {
    path: Vec<String>,
    sport: String,
    device: String,
    id: String,
    stats: TrackStats,
    laps: Laps,
    point: Option<Point>,
    in_activity: bool,
    parsed: Parsed,
}

impl State {
    fn ancestor(&self, up: usize) -> &str {
        let n = self.path.len();
        if n > up {
            self.path[n - 1 - up].as_str()
        } else {
            ""
        }
    }

    fn open(&mut self, name: String, attrs: &[(String, String)]) {
        match name.as_str() {
            "Activity" => {
                self.in_activity = true;
                self.sport = attr(attrs, "Sport").unwrap_or_default().to_string();
                self.id.clear();
                self.device.clear();
                self.stats = TrackStats::default();
                self.laps = Laps::default();
            }
            "Trackpoint" if self.in_activity => self.point = Some(Point::default()),
            _ => {}
        }
        self.path.push(name);
    }

    fn number(text: &str) -> Option<f64> {
        text.parse::<f64>().ok().filter(|v| v.is_finite())
    }

    fn text(&mut self, text: &str) {
        let Some(current) = self.path.last().cloned() else {
            return;
        };
        let parent = self.ancestor(1).to_string();
        let grand = self.ancestor(2).to_string();
        if let Some(point) = self.point.as_mut() {
            match (current.as_str(), parent.as_str()) {
                ("Time", "Trackpoint") => point.time = time_of(text),
                ("AltitudeMeters", "Trackpoint") => point.elevation = Self::number(text),
                ("DistanceMeters", "Trackpoint") => point.distance = Self::number(text),
                ("LatitudeDegrees", "Position") => point.lat = Self::number(text),
                ("LongitudeDegrees", "Position") => point.lon = Self::number(text),
                ("Value", "HeartRateBpm") => point.heart_rate = Self::number(text),
                _ => {}
            }
            return;
        }
        if !self.in_activity {
            return;
        }
        match (current.as_str(), parent.as_str(), grand.as_str()) {
            ("Id", "Activity", _) => self.id = text.to_string(),
            ("Name", "Creator", "Activity") => self.device = text.to_string(),
            ("TotalTimeSeconds", "Lap", _) => {
                self.laps.seconds += Self::number(text).unwrap_or(0.0);
            }
            ("DistanceMeters", "Lap", _) => {
                self.laps.distance += Self::number(text).unwrap_or(0.0);
            }
            ("Calories", "Lap", _) => self.laps.calories += Self::number(text).unwrap_or(0.0),
            ("Value", "AverageHeartRateBpm", "Lap") => {
                if let Some(v) = Self::number(text) {
                    self.laps.hr_average_sum += v;
                    self.laps.hr_average_count += 1;
                }
            }
            ("Value", "MaximumHeartRateBpm", "Lap") => {
                if let Some(v) = Self::number(text) {
                    self.laps.hr_max = Some(self.laps.hr_max.map_or(v, |m| m.max(v)));
                }
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str) {
        self.path.pop();
        match name {
            "Trackpoint" => {
                if let Some(point) = self.point.take() {
                    self.stats.add(&point);
                }
            }
            "Activity" => {
                self.in_activity = false;
                self.finish_activity();
            }
            _ => {}
        }
    }

    fn summary(&self) -> Option<Summary> {
        if let Some(s) = self.stats.finish() {
            return Some(s);
        }
        let start = time_of(&self.id)?;
        if self.laps.seconds <= 0.0 {
            return None;
        }
        Some(Summary {
            start,
            end: start + Duration::seconds(self.laps.seconds.round() as i64),
            timed_points: 0,
            distance: self.laps.distance,
            climb: 0.0,
            hr_average: None,
            hr_min: None,
            hr_max: None,
        })
    }

    fn finish_activity(&mut self) {
        let Some(mut summary) = self.summary() else {
            self.parsed
                .skipped
                .add("activité : ni points horodatés ni tours chronométrés");
            return;
        };
        if summary.hr_average.is_none() && self.laps.hr_average_count > 0 {
            let average = self.laps.hr_average_sum / f64::from(self.laps.hr_average_count);
            summary.hr_average = Some(average.round() as i64);
            summary.hr_max = self.laps.hr_max.map(|v| v.round() as i64);
        }
        let distance = if self.laps.distance > 0.0 {
            self.laps.distance
        } else {
            summary.distance
        };
        self.parsed.rows.push(workout_row(&Draft {
            prefix: "tcx",
            kind: "tcx",
            sport: &self.sport,
            device: &self.device,
            summary,
            duration: Some(self.laps.seconds.round() as i64),
            distance: Some(distance),
            calories: Some(self.laps.calories),
        }));
    }
}

pub fn parse<R: BufRead>(source: R, cancel: &AtomicBool) -> Result<Parsed, AppError> {
    let mut state = State::default();
    let cancelled = scan(source, WHAT, "TrainingCenterDatabase", cancel, |tok| {
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
            .add("activité : aucune activité dans le fichier");
    }
    Ok(state.parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const FILE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<TrainingCenterDatabase xmlns="http://www.garmin.com/xmlschemas/TrainingCenterDatabase/v2">
 <Activities>
  <Activity Sport="Biking">
   <Id>2026-10-25T08:00:00Z</Id>
   <Lap StartTime="2026-10-25T08:00:00Z">
    <TotalTimeSeconds>1200</TotalTimeSeconds><DistanceMeters>6000</DistanceMeters><Calories>180</Calories>
    <AverageHeartRateBpm><Value>130</Value></AverageHeartRateBpm><MaximumHeartRateBpm><Value>160</Value></MaximumHeartRateBpm>
    <Track>
     <Trackpoint><Time>2026-10-25T08:00:00Z</Time><Position><LatitudeDegrees>48.0</LatitudeDegrees><LongitudeDegrees>2.0</LongitudeDegrees></Position><AltitudeMeters>50</AltitudeMeters><DistanceMeters>0</DistanceMeters><HeartRateBpm><Value>110</Value></HeartRateBpm></Trackpoint>
     <Trackpoint><Time>2026-10-25T08:10:00Z</Time><AltitudeMeters>60</AltitudeMeters><DistanceMeters>3000</DistanceMeters><HeartRateBpm><Value>150</Value></HeartRateBpm></Trackpoint>
     <Trackpoint><Time>2026-10-25T08:20:00Z</Time><AltitudeMeters>58</AltitudeMeters><DistanceMeters>6000</DistanceMeters><HeartRateBpm><Value>130</Value></HeartRateBpm></Trackpoint>
    </Track>
   </Lap>
   <Creator><Name>Compteur Test</Name></Creator>
  </Activity>
  <Activity Sport="Other">
   <Id>2026-10-25T18:00:00Z</Id>
   <Lap StartTime="2026-10-25T18:00:00Z"><TotalTimeSeconds>900</TotalTimeSeconds><DistanceMeters>0</DistanceMeters><Calories>70</Calories><AverageHeartRateBpm><Value>95</Value></AverageHeartRateBpm><MaximumHeartRateBpm><Value>120</Value></MaximumHeartRateBpm></Lap>
  </Activity>
 </Activities>
</TrainingCenterDatabase>"#;

    fn parse_str(xml: impl AsRef<[u8]>) -> Result<Parsed, AppError> {
        parse(Cursor::new(xml.as_ref()), &AtomicBool::new(false))
    }

    #[test]
    fn an_activity_with_a_track_takes_laps_for_totals_and_points_for_the_rest() {
        let parsed = parse_str(FILE).expect("tcx");
        assert_eq!(parsed.rows.len(), 2);
        let ride = &parsed.rows[0];
        assert_eq!(ride.category_label, "Velo");
        assert_eq!(ride.device, "Compteur Test");
        assert_eq!(ride.duration, 1200);
        assert_eq!(ride.distance, Some(6000.0));
        assert_eq!(ride.calories, Some(180.0));
        assert_eq!(ride.elevation, Some(10.0));
        assert_eq!(ride.hr_average, Some(130));
        assert_eq!(ride.hr_min, Some(110));
        assert_eq!(ride.hr_max, Some(150));
    }

    #[test]
    fn an_activity_without_a_track_falls_back_on_its_laps() {
        let parsed = parse_str(FILE).expect("tcx");
        let indoor = &parsed.rows[1];
        assert_eq!(indoor.category_label, "Activite");
        assert_eq!(indoor.start_at, "2026-10-25T18:00:00Z");
        assert_eq!(indoor.end_at, "2026-10-25T18:15:00Z");
        assert_eq!(indoor.duration, 900);
        assert_eq!(indoor.distance, None);
        assert_eq!(indoor.calories, Some(70.0));
        assert_eq!(indoor.hr_average, Some(95));
        assert_eq!(indoor.hr_max, Some(120));
        assert_eq!(parsed.skipped.total(), 0);
    }

    #[test]
    fn a_truncated_tcx_and_a_gpx_given_as_tcx_are_named_errors() {
        let err = parse_str(&FILE.as_bytes()[..FILE.len() - 60]).unwrap_err();
        assert!(
            err.to_string().contains("tronqué") || err.to_string().contains("illisible"),
            "{err}"
        );
        let err = parse_str("<gpx></gpx>").unwrap_err();
        assert!(err.to_string().contains("TrainingCenterDatabase"), "{err}");
    }
}
