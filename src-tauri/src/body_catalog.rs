use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    pub view: String,
    pub x: f64,
    pub y: f64,
    pub label_side: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyMeasurement {
    pub slug: String,
    pub label: String,
    pub step_label: String,
    pub zone: String,
    pub nature: String,
    pub side: String,
    pub pair_key: Option<String>,
    pub order: i64,
    pub is_key: bool,
    pub min: f64,
    pub max: f64,
    pub guidance: String,
    pub anchor: Option<Anchor>,
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    measurements: Vec<BodyMeasurement>,
}

// Source unique partagee avec le frontend : le meme fichier est importe par Vite
// cote React. Ne pas dupliquer ce JSON dans src-tauri.
const CATALOG_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/features/bodyProfile/catalog.json"
));

static CATALOG: OnceLock<Vec<BodyMeasurement>> = OnceLock::new();

pub fn catalog() -> &'static [BodyMeasurement] {
    CATALOG.get_or_init(|| {
        let parsed: CatalogFile =
            serde_json::from_str(CATALOG_JSON).expect("catalog.json invalide");
        parsed.measurements
    })
}

pub fn find(slug: &str) -> Option<&'static BodyMeasurement> {
    catalog().iter().find(|m| m.slug == slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_catalog_from_json() {
        let all = catalog();
        assert!(!all.is_empty(), "le catalogue ne doit pas etre vide");
        let height = find("height").expect("height doit exister");
        assert_eq!(height.label, "Hauteur totale");
        assert_eq!(height.min, 100.0);
        assert_eq!(height.max, 250.0);
        assert!(height.is_key);
        assert_eq!(height.pair_key, None);
    }

    #[test]
    fn maps_camel_case_fields() {
        let left = find("bicep_left").expect("bicep_left doit exister");
        assert_eq!(left.pair_key.as_deref(), Some("bicep"));
        assert_eq!(left.step_label, "Tour de bras");
        assert_eq!(left.side, "left");
    }

    #[test]
    fn returns_none_for_unknown_slug() {
        assert!(find("slug_inexistant").is_none());
    }

    #[test]
    fn catalog_has_47_measurements() {
        assert_eq!(catalog().len(), 47);
    }

    #[test]
    fn slugs_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for m in catalog() {
            assert!(seen.insert(&m.slug), "slug duplique : {}", m.slug);
        }
    }

    #[test]
    fn bounds_are_coherent() {
        for m in catalog() {
            assert!(m.min < m.max, "bornes incoherentes pour {}", m.slug);
            assert!(m.min >= 0.0, "borne min negative pour {}", m.slug);
        }
    }

    #[test]
    fn key_measurements_count_is_22() {
        let keys = catalog().iter().filter(|m| m.is_key).count();
        assert_eq!(keys, 22);
    }

    #[test]
    fn pairs_have_exactly_two_sides() {
        use std::collections::HashMap;
        let mut pairs: HashMap<&str, Vec<&str>> = HashMap::new();
        for m in catalog() {
            if let Some(pk) = &m.pair_key {
                pairs.entry(pk).or_default().push(&m.side);
            }
        }
        assert_eq!(pairs.len(), 14, "14 paires laterisees attendues");
        for (pk, sides) in pairs {
            assert_eq!(sides.len(), 2, "la paire {pk} doit avoir 2 cotes");
            assert!(
                sides.contains(&"left"),
                "la paire {pk} doit avoir un cote gauche"
            );
            assert!(
                sides.contains(&"right"),
                "la paire {pk} doit avoir un cote droit"
            );
        }
    }

    #[test]
    fn singles_have_no_pair_key() {
        for m in catalog() {
            if m.side == "none" {
                assert!(
                    m.pair_key.is_none(),
                    "{} ne doit pas avoir de pairKey",
                    m.slug
                );
            }
        }
    }

    #[test]
    fn orders_are_unique_and_sorted() {
        let mut orders: Vec<i64> = catalog().iter().map(|m| m.order).collect();
        let count = orders.len();
        orders.sort_unstable();
        orders.dedup();
        assert_eq!(orders.len(), count, "les ordres doivent etre uniques");
    }

    #[test]
    fn zones_are_known() {
        let known = ["global", "neck_shoulders", "torso", "arms", "legs"];
        for m in catalog() {
            assert!(
                known.contains(&m.zone.as_str()),
                "zone inconnue : {}",
                m.zone
            );
        }
    }

    #[test]
    fn natures_are_known() {
        let known = ["circumference", "length", "width", "height"];
        for m in catalog() {
            assert!(
                known.contains(&m.nature.as_str()),
                "nature inconnue : {}",
                m.nature
            );
        }
    }

    #[test]
    fn every_measurement_has_guidance() {
        for m in catalog() {
            assert!(
                !m.guidance.trim().is_empty(),
                "guidance manquante pour {}",
                m.slug
            );
        }
    }

    // Une paire laterisee ne porte qu'une seule ancre, cote gauche : le PDF
    // annote une fois et affiche les deux valeurs dans le meme libelle.
    // L'invariant n'est donc pas "toute mesure cle a une ancre" mais
    // "toute mesure cle qui n'est pas le cote droit d'une paire en a une".
    #[test]
    fn every_anchorable_key_has_an_anchor() {
        for m in catalog().iter().filter(|m| m.is_key && m.side != "right") {
            assert!(
                m.anchor.is_some(),
                "ancre manquante pour la mesure cle {}",
                m.slug
            );
        }
    }

    #[test]
    fn anchors_stay_inside_the_figure_box() {
        for m in catalog() {
            let Some(anchor) = &m.anchor else { continue };
            assert!(
                anchor.view == "front" || anchor.view == "back",
                "vue invalide pour {} : {}",
                m.slug,
                anchor.view
            );
            assert!(
                anchor.label_side == "left" || anchor.label_side == "right",
                "cote de libelle invalide pour {}",
                m.slug
            );
            assert!(
                (0.0..=100.0).contains(&anchor.x) && (0.0..=100.0).contains(&anchor.y),
                "ancre hors boite pour {}",
                m.slug
            );
        }
    }

    #[test]
    fn paired_keys_carry_a_single_anchor() {
        use std::collections::HashSet;
        let mut anchored_pairs: HashSet<&str> = HashSet::new();
        for m in catalog().iter().filter(|m| m.is_key && m.anchor.is_some()) {
            if let Some(pair) = &m.pair_key {
                assert!(
                    anchored_pairs.insert(pair),
                    "la paire {pair} ne doit porter qu'une seule ancre"
                );
            }
        }
    }

    // Deux libelles trop proches sur la meme marge se chevauchent sur le PDF.
    // 3 % de la hauteur de figure valent environ 5 mm, soit une ligne de texte.
    #[test]
    fn anchor_labels_do_not_overlap() {
        use std::collections::HashMap;
        let mut margins: HashMap<(&str, &str), Vec<(&str, f64)>> = HashMap::new();
        for m in catalog() {
            let Some(anchor) = &m.anchor else { continue };
            margins
                .entry((anchor.view.as_str(), anchor.label_side.as_str()))
                .or_default()
                .push((m.slug.as_str(), anchor.y));
        }
        for ((view, side), mut labels) in margins {
            labels.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            for pair in labels.windows(2) {
                let gap = pair[1].1 - pair[0].1;
                assert!(
                    gap >= 3.0,
                    "libelles trop proches en marge {view}/{side} : {} et {} separes de {gap:.1} %",
                    pair[0].0,
                    pair[1].0
                );
            }
        }
    }
}
