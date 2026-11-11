#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawRecord {
    pub kind: String,
    pub unit: String,
    pub value: String,
    pub origin: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawStat {
    pub kind: String,
    pub sum: String,
    pub unit: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawWorkout {
    pub activity: String,
    pub origin: String,
    pub start: String,
    pub end: String,
    pub duration: String,
    pub duration_unit: String,
    pub distance: String,
    pub distance_unit: String,
    pub energy: String,
    pub energy_unit: String,
    pub stats: Vec<RawStat>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Record(RawRecord),
    Workout(RawWorkout),
}
