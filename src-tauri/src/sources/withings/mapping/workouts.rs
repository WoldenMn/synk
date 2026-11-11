pub fn workout_category_label(category: i64) -> &'static str {
    match category {
        1 => "Marche",
        2 => "Course",
        3 => "Randonnee",
        4 => "Patinage",
        5 => "BMX",
        6 => "Velo",
        7 => "Natation",
        8 => "Surf",
        9 => "Kitesurf",
        10 => "Planche a voile",
        11 => "Bodyboard",
        12 => "Tennis",
        13 => "Tennis de table",
        14 => "Squash",
        15 => "Badminton",
        16 => "Musculation",
        17 => "Calisthenics",
        18 => "Elliptique",
        19 => "Pilates",
        20 => "Basketball",
        21 => "Football",
        22 => "Rugby",
        23 => "Volleyball",
        24 => "Waterpolo",
        25 => "Equitation",
        26 => "Golf",
        27 => "Yoga",
        28 => "Danse",
        29 => "Boxe",
        30 => "Escrime",
        31 => "Gymnastique",
        32 => "Voile",
        33 => "Aviron",
        34 => "Plongee",
        35 => "Ski alpin",
        36 => "Autre",
        128 => "Pas d'activite",
        187 => "Rameur",
        188 => "Zumba",
        190 => "Cricket",
        191 => "Baseball",
        192 => "Handball",
        193 => "Hockey",
        195 => "Escalade",
        196 => "Tapis de course",
        197 => "Velo d'interieur",
        198 => "Elliptique (interieur)",
        200 => "Marche nordique",
        260 => "Ski de fond",
        261 => "Snowboard",
        262 => "Raquettes",
        263 => "Patinage sur glace",
        264 => "Roller",
        266 => "HIIT",
        272 => "Stretching",
        307 => "Corde a sauter",
        _ => "Autre",
    }
}

#[cfg(test)]
mod helpers_tests {
    use super::*;

    #[test]
    fn workout_category_label_known() {
        assert_eq!(workout_category_label(1), "Marche");
        assert_eq!(workout_category_label(2), "Course");
        assert_eq!(workout_category_label(6), "Velo");
        assert_eq!(workout_category_label(266), "HIIT");
    }

    #[test]
    fn workout_category_label_unknown_falls_back_to_autre() {
        assert_eq!(workout_category_label(9999), "Autre");
        assert_eq!(workout_category_label(0), "Autre");
    }
}
