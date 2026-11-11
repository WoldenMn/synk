pub enum Bound {
    F(f64, f64),
    I(i64, i64),
}

pub struct ColumnBound {
    pub table: &'static str,
    pub column: &'static str,
    pub label: &'static str,
    pub bound: Bound,
}

pub const BOUNDS: &[ColumnBound] = &[
    ColumnBound {
        table: "body_measures_legacy",
        column: "weight",
        label: "Poids",
        bound: Bound::F(0.0, 500.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "fat_ratio",
        label: "Masse grasse (%)",
        bound: Bound::F(0.0, 100.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "fat_mass",
        label: "Masse grasse (kg)",
        bound: Bound::F(0.0, 500.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "fat_free_mass",
        label: "Masse maigre",
        bound: Bound::F(0.0, 500.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "muscle_mass",
        label: "Masse musculaire",
        bound: Bound::F(0.0, 500.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "bone_mass",
        label: "Masse osseuse",
        bound: Bound::F(0.0, 100.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "hydration",
        label: "Hydratation (kg)",
        bound: Bound::F(0.0, 300.0),
    },
    ColumnBound {
        table: "body_measures_legacy",
        column: "bmi",
        label: "IMC",
        bound: Bound::F(0.0, 100.0),
    },
    ColumnBound {
        table: "blood_pressure",
        column: "systolic",
        label: "Systolique",
        bound: Bound::I(0, 300),
    },
    ColumnBound {
        table: "blood_pressure",
        column: "diastolic",
        label: "Diastolique",
        bound: Bound::I(0, 200),
    },
    ColumnBound {
        table: "blood_pressure",
        column: "heart_rate",
        label: "Frequence cardiaque",
        bound: Bound::I(0, 300),
    },
    ColumnBound {
        table: "sleep_records",
        column: "duration_total",
        label: "Duree de sommeil",
        bound: Bound::I(0, 86400),
    },
    ColumnBound {
        table: "sleep_records",
        column: "sleep_score",
        label: "Score de sommeil",
        bound: Bound::I(0, 100),
    },
    ColumnBound {
        table: "activity",
        column: "steps",
        label: "Pas",
        bound: Bound::I(0, 200_000),
    },
    ColumnBound {
        table: "activity",
        column: "distance",
        label: "Distance",
        bound: Bound::F(0.0, 1_000_000.0),
    },
    ColumnBound {
        table: "activity",
        column: "calories_active",
        label: "Calories actives",
        bound: Bound::F(0.0, 100_000.0),
    },
    ColumnBound {
        table: "workouts",
        column: "duration",
        label: "Duree workout",
        bound: Bound::I(0, 86400),
    },
    ColumnBound {
        table: "workouts",
        column: "distance",
        label: "Distance workout",
        bound: Bound::F(0.0, 1_000_000.0),
    },
    ColumnBound {
        table: "workouts",
        column: "calories",
        label: "Calories workout",
        bound: Bound::F(0.0, 100_000.0),
    },
    ColumnBound {
        table: "workouts",
        column: "hr_average",
        label: "FC moyenne workout",
        bound: Bound::I(0, 300),
    },
    ColumnBound {
        table: "heart_records",
        column: "heart_rate",
        label: "Frequence cardiaque ECG",
        bound: Bound::I(0, 300),
    },
    ColumnBound {
        table: "manual_measurements_legacy",
        column: "value",
        label: "Valeur",
        bound: Bound::F(-1_000_000.0, 1_000_000.0),
    },
    ColumnBound {
        table: "user_profile",
        column: "age",
        label: "Age",
        bound: Bound::I(0, 150),
    },
    ColumnBound {
        table: "user_profile",
        column: "height_cm",
        label: "Taille",
        bound: Bound::F(0.0, 300.0),
    },
];
