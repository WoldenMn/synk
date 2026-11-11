use chrono::{DateTime, Utc};

const EARTH_RADIUS_M: f64 = 6_371_008.8;
const CLIMB_STEP_M: f64 = 3.0;

#[derive(Debug, Clone, Default)]
pub struct Point {
    pub time: Option<DateTime<Utc>>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub elevation: Option<f64>,
    pub heart_rate: Option<f64>,
    pub distance: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub timed_points: u64,
    pub distance: f64,
    pub climb: f64,
    pub hr_average: Option<i64>,
    pub hr_min: Option<i64>,
    pub hr_max: Option<i64>,
}

#[derive(Debug, Default)]
pub struct TrackStats {
    first: Option<DateTime<Utc>>,
    last: Option<DateTime<Utc>>,
    timed: u64,
    walked: f64,
    reported: Option<f64>,
    previous: Option<(f64, f64)>,
    reference: Option<f64>,
    climb: f64,
    hr_sum: f64,
    hr_count: u64,
    hr_min: Option<f64>,
    hr_max: Option<f64>,
}

pub fn haversine(a: (f64, f64), b: (f64, f64)) -> f64 {
    let (lat1, lon1) = (a.0.to_radians(), a.1.to_radians());
    let (lat2, lon2) = (b.0.to_radians(), b.1.to_radians());
    let h = ((lat2 - lat1) / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * ((lon2 - lon1) / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * h.sqrt().asin()
}

fn valid_position(lat: f64, lon: f64) -> bool {
    (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon)
}

impl TrackStats {
    pub fn add(&mut self, p: &Point) {
        if let Some(t) = p.time {
            self.first = Some(self.first.map_or(t, |f| f.min(t)));
            self.last = Some(self.last.map_or(t, |l| l.max(t)));
            self.timed += 1;
        }
        if let (Some(lat), Some(lon)) = (p.lat, p.lon) {
            if valid_position(lat, lon) {
                if let Some(prev) = self.previous {
                    self.walked += haversine(prev, (lat, lon));
                }
                self.previous = Some((lat, lon));
            }
        }
        if let Some(d) = p.distance.filter(|d| d.is_finite() && *d >= 0.0) {
            self.reported = Some(self.reported.map_or(d, |r| r.max(d)));
        }
        if let Some(e) = p.elevation.filter(|e| e.is_finite()) {
            match self.reference {
                None => self.reference = Some(e),
                Some(r) if e - r >= CLIMB_STEP_M => {
                    self.climb += e - r;
                    self.reference = Some(e);
                }
                Some(r) if r - e >= CLIMB_STEP_M => self.reference = Some(e),
                Some(_) => {}
            }
        }
        if let Some(hr) = p.heart_rate.filter(|h| (30.0..=250.0).contains(h)) {
            self.hr_sum += hr;
            self.hr_count += 1;
            self.hr_min = Some(self.hr_min.map_or(hr, |m| m.min(hr)));
            self.hr_max = Some(self.hr_max.map_or(hr, |m| m.max(hr)));
        }
    }

    pub fn finish(&self) -> Option<Summary> {
        let (start, end) = (self.first?, self.last?);
        if self.timed < 2 || end <= start {
            return None;
        }
        let rounded = |v: f64| v.round() as i64;
        Some(Summary {
            start,
            end,
            timed_points: self.timed,
            distance: self.reported.unwrap_or(self.walked),
            climb: self.climb,
            hr_average: (self.hr_count > 0).then(|| rounded(self.hr_sum / self.hr_count as f64)),
            hr_min: self.hr_min.map(rounded),
            hr_max: self.hr_max.map(rounded),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 25, 16, minute, 0).unwrap()
    }

    fn point(minute: u32, lat: f64, ele: f64, hr: f64) -> Point {
        Point {
            time: Some(at(minute)),
            lat: Some(lat),
            lon: Some(2.0),
            elevation: Some(ele),
            heart_rate: Some(hr),
            distance: None,
        }
    }

    #[test]
    fn one_degree_of_latitude_is_about_111_kilometres() {
        let d = haversine((48.0, 2.0), (49.0, 2.0));
        assert!((d - 111_195.0).abs() < 50.0, "{d}");
    }

    #[test]
    fn a_track_gives_its_span_distance_climb_and_heart_rate() {
        let mut s = TrackStats::default();
        s.add(&point(0, 48.000, 100.0, 120.0));
        s.add(&point(5, 48.001, 110.0, 140.0));
        s.add(&point(10, 48.002, 105.0, 160.0));
        let sum = s.finish().expect("résumé");
        assert_eq!(sum.start, at(0));
        assert_eq!(sum.end, at(10));
        assert!((sum.distance - 222.4).abs() < 1.0, "{}", sum.distance);
        assert_eq!(sum.climb, 10.0);
        assert_eq!(sum.hr_average, Some(140));
        assert_eq!(sum.hr_min, Some(120));
        assert_eq!(sum.hr_max, Some(160));
    }

    #[test]
    fn gps_noise_under_the_step_is_not_counted_as_climb() {
        let mut s = TrackStats::default();
        for (i, e) in [100.0, 101.0, 100.0, 102.0, 100.5].iter().enumerate() {
            s.add(&point(i as u32, 48.0, *e, 100.0));
        }
        assert_eq!(s.finish().expect("résumé").climb, 0.0);
    }

    #[test]
    fn a_reported_distance_wins_over_the_walked_one() {
        let mut s = TrackStats::default();
        let mut p = point(0, 48.0, 100.0, 100.0);
        p.distance = Some(0.0);
        s.add(&p);
        let mut q = point(5, 48.01, 100.0, 100.0);
        q.distance = Some(1500.0);
        s.add(&q);
        assert_eq!(s.finish().expect("résumé").distance, 1500.0);
    }

    #[test]
    fn fewer_than_two_timed_points_give_no_summary() {
        let mut s = TrackStats::default();
        assert!(s.finish().is_none());
        s.add(&point(0, 48.0, 100.0, 100.0));
        assert!(s.finish().is_none());
        s.add(&Point {
            lat: Some(48.1),
            lon: Some(2.0),
            ..Point::default()
        });
        assert!(s.finish().is_none());
    }

    #[test]
    fn impossible_positions_and_heart_rates_are_ignored() {
        let mut s = TrackStats::default();
        let mut a = point(0, 48.0, 100.0, 400.0);
        a.lon = Some(2.0);
        s.add(&a);
        let mut b = point(5, 95.0, 100.0, 10.0);
        b.lon = Some(2.0);
        s.add(&b);
        let sum = s.finish().expect("résumé");
        assert_eq!(sum.distance, 0.0);
        assert_eq!(sum.hr_average, None);
    }
}
