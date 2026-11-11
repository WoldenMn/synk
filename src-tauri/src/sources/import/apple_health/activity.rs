use std::collections::BTreeMap;

use crate::sources::import::rows::ActivityRow;

#[derive(Default, Clone, Copy)]
pub struct DayTotals {
    pub steps: f64,
    pub distance: f64,
    pub active: f64,
    pub basal: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    Steps,
    Distance,
    Active,
    Basal,
}

#[derive(Default)]
pub struct ActivityAgg {
    days: BTreeMap<String, BTreeMap<String, DayTotals>>,
}

impl ActivityAgg {
    pub fn add(&mut self, day: &str, origin: &str, quantity: Quantity, value: f64) {
        let totals = self
            .days
            .entry(day.to_string())
            .or_default()
            .entry(origin.to_string())
            .or_default();
        match quantity {
            Quantity::Steps => totals.steps += value,
            Quantity::Distance => totals.distance += value,
            Quantity::Active => totals.active += value,
            Quantity::Basal => totals.basal += value,
        }
    }

    pub fn rows(self) -> Vec<ActivityRow> {
        self.days
            .into_iter()
            .filter_map(|(day, origins)| day_row(day, &origins))
            .collect()
    }
}

fn best(
    origins: &BTreeMap<String, DayTotals>,
    pick: impl Fn(&DayTotals) -> f64,
) -> Option<(&str, f64)> {
    origins
        .iter()
        .map(|(name, t)| (name.as_str(), pick(t)))
        .filter(|(_, v)| *v > 0.0)
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

fn day_row(day: String, origins: &BTreeMap<String, DayTotals>) -> Option<ActivityRow> {
    let steps = best(origins, |t| t.steps);
    let distance = best(origins, |t| t.distance);
    let active = best(origins, |t| t.active);
    let basal = best(origins, |t| t.basal);
    let device = steps
        .or(distance)
        .or(active)
        .or(basal)
        .map(|(name, _)| name.to_string())?;
    let calories_total = match (active, basal) {
        (Some((_, a)), Some((_, b))) => Some(a + b),
        _ => None,
    };
    Some(ActivityRow {
        external_id: format!("ah:activity:{day}"),
        date: day,
        steps: steps.map(|(_, v)| v.round() as i64),
        distance: distance.map(|(_, v)| v),
        calories_active: active.map(|(_, v)| v),
        calories_total,
        device,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_day_takes_the_fullest_origin_of_each_quantity_and_never_adds_two_origins() {
        let mut agg = ActivityAgg::default();
        agg.add("2026-10-25", "iPhone", Quantity::Steps, 4000.0);
        agg.add("2026-10-25", "iPhone", Quantity::Steps, 2000.0);
        agg.add("2026-10-25", "Apple Watch", Quantity::Steps, 6100.0);
        agg.add("2026-10-25", "Apple Watch", Quantity::Active, 410.0);
        agg.add("2026-10-25", "Apple Watch", Quantity::Basal, 1700.0);
        agg.add("2026-10-25", "iPhone", Quantity::Distance, 4800.0);
        let rows = agg.rows();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.steps, Some(6100));
        assert_eq!(r.device, "Apple Watch");
        assert_eq!(r.distance, Some(4800.0));
        assert_eq!(r.calories_active, Some(410.0));
        assert_eq!(r.calories_total, Some(2110.0));
        assert_eq!(r.external_id, "ah:activity:2026-10-25");
    }

    #[test]
    fn a_day_without_any_positive_quantity_gives_no_row() {
        let mut agg = ActivityAgg::default();
        agg.add("2026-10-25", "iPhone", Quantity::Steps, 0.0);
        assert!(agg.rows().is_empty());
    }

    #[test]
    fn days_come_out_in_date_order() {
        let mut agg = ActivityAgg::default();
        agg.add("2026-10-26", "a", Quantity::Steps, 1.0);
        agg.add("2026-10-24", "a", Quantity::Steps, 1.0);
        let dates: Vec<String> = agg.rows().into_iter().map(|r| r.date).collect();
        assert_eq!(dates, vec!["2026-10-24", "2026-10-26"]);
    }
}
