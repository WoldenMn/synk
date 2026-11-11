use super::callouts::{sheet_layout, Label, Side, LABEL_PT, NAME_RISE_MM, VALUE_DROP_MM};
use super::canvas::Canvas;
use super::figures::figures;
use super::sheet::{dpi_for_width, format_value, value_for};
use super::text_metrics::TextMetrics;
use crate::body_catalog;
use crate::commands::body_profile::SheetRow;
use crate::error::AppError;
use printpdf::{BuiltinFont, Color, Mm, Rgb, XObjectTransform};

const LEADER_GREY: f32 = 0.45;
const DOT_GREY: f32 = 0.25;
const DOT_RADIUS_MM: f32 = 0.6;

fn grey(level: f32) -> Color {
    Color::Rgb(Rgb::new(level, level, level, None))
}

fn callout_name(m: &body_catalog::BodyMeasurement) -> String {
    match &m.pair_key {
        Some(_) => format!("{} G/D", m.step_label),
        None => m.step_label.clone(),
    }
}

fn callout_value(m: &body_catalog::BodyMeasurement, rows: &[SheetRow]) -> String {
    match &m.pair_key {
        Some(pair) => format!(
            "{} / {} cm",
            format_value(value_for(rows, &format!("{pair}_left"))),
            format_value(value_for(rows, &format!("{pair}_right")))
        ),
        None => format!("{} cm", format_value(value_for(rows, &m.slug))),
    }
}

fn callout_labels(rows: &[SheetRow]) -> Vec<Label> {
    body_catalog::catalog()
        .iter()
        .filter(|m| m.is_key)
        .filter_map(|m| {
            let anchor = m.anchor.as_ref()?;
            Some(Label {
                view: anchor.view.clone(),
                side: if anchor.label_side == "left" {
                    Side::Left
                } else {
                    Side::Right
                },
                x_pct: anchor.x as f32,
                y_pct: anchor.y as f32,
                name: callout_name(m),
                value: callout_value(m, rows),
            })
        })
        .collect()
}

pub(super) fn draw_figures(canvas: &mut Canvas, rows: &[SheetRow]) -> Result<f32, AppError> {
    let metrics = TextMetrics::helvetica()?;
    let [front, back] = figures()?;
    let aspect = front.height as f32 / front.width as f32;
    let page = sheet_layout(&callout_labels(rows), aspect, &metrics);
    for (figure, frame) in [&front, &back].into_iter().zip(page.figures) {
        canvas.place_image(
            figure,
            XObjectTransform {
                translate_x: Some(Mm(frame.left).into()),
                translate_y: Some(Mm(frame.bottom).into()),
                dpi: Some(dpi_for_width(figure.width as u32, frame.width())),
                ..Default::default()
            },
        );
    }
    canvas.set_outline(0.4, grey(LEADER_GREY));
    for c in &page.callouts {
        canvas.add_polyline(&c.leader);
    }
    let anchors: Vec<(f32, f32)> = page.callouts.iter().map(|c| c.anchor).collect();
    canvas.fill_discs(&anchors, DOT_RADIUS_MM, grey(DOT_GREY));
    for c in &page.callouts {
        canvas.use_text(
            c.name.as_str(),
            LABEL_PT,
            Mm(c.text_x),
            Mm(c.line_y + NAME_RISE_MM),
            BuiltinFont::Helvetica,
        );
        canvas.use_text(
            c.value.as_str(),
            LABEL_PT,
            Mm(c.text_x),
            Mm(c.line_y - VALUE_DROP_MM),
            BuiltinFont::HelveticaBold,
        );
    }
    Ok(page
        .callouts
        .iter()
        .map(|c| c.text_box.left)
        .chain(page.figures.iter().map(|f| f.left))
        .fold(f32::INFINITY, f32::min))
}

#[cfg(test)]
mod tests {
    use super::super::callouts::{
        anchor_page_y, Callout, Rect, SheetLayout, AREA_BOTTOM_MM, AREA_TOP_MM, SIDE_MARGIN_MM,
    };
    use super::super::layout::PAGE_WIDTH_MM;
    use super::*;

    fn row(slug: &str, value: Option<f64>) -> SheetRow {
        SheetRow {
            slug: slug.to_string(),
            label: slug.to_string(),
            unit: "cm".to_string(),
            value,
            measured_at: None,
        }
    }

    fn full_rows() -> Vec<SheetRow> {
        body_catalog::catalog()
            .iter()
            .map(|m| row(&m.slug, Some(199.9)))
            .collect()
    }

    fn png_aspect() -> f32 {
        let [front, _] = figures().unwrap();
        front.height as f32 / front.width as f32
    }

    fn layout_for(rows: &[SheetRow]) -> SheetLayout {
        sheet_layout(
            &callout_labels(rows),
            png_aspect(),
            &TextMetrics::helvetica().unwrap(),
        )
    }

    fn callout<'a>(page: &'a SheetLayout, slug: &str) -> &'a Callout {
        let name = callout_name(body_catalog::find(slug).unwrap());
        page.callouts
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("aucun libelle pour {slug}"))
    }

    fn frame_of<'a>(page: &'a SheetLayout, slug: &str) -> &'a Rect {
        let view = &body_catalog::find(slug)
            .unwrap()
            .anchor
            .as_ref()
            .unwrap()
            .view;
        &page.figures[usize::from(view == "back")]
    }

    #[test]
    fn places_one_callout_per_anchored_key() {
        assert_eq!(layout_for(&[]).callouts.len(), 16);
        assert_eq!(layout_for(&full_rows()).callouts.len(), 16);
    }

    #[test]
    fn keeps_every_label_off_the_figures_apart_and_on_the_page() {
        for rows in [Vec::new(), full_rows()] {
            let page = layout_for(&rows);
            for (i, c) in page.callouts.iter().enumerate() {
                for frame in &page.figures {
                    assert!(!c.text_box.overlaps(frame), "{} touche une figure", c.name);
                }
                for other in &page.callouts[i + 1..] {
                    assert!(
                        !c.text_box.overlaps(&other.text_box),
                        "{} chevauche {}",
                        c.name,
                        other.name
                    );
                }
                let b = c.text_box;
                assert!(
                    b.left >= SIDE_MARGIN_MM - 1e-3,
                    "{} deborde a gauche",
                    c.name
                );
                assert!(
                    b.right <= PAGE_WIDTH_MM - SIDE_MARGIN_MM + 1e-3,
                    "{} deborde a droite",
                    c.name
                );
                assert!(
                    b.bottom >= AREA_BOTTOM_MM && b.top <= AREA_TOP_MM,
                    "{}",
                    c.name
                );
            }
        }
    }

    #[test]
    fn draws_both_figures_large_and_undistorted() {
        let page = layout_for(&full_rows());
        for frame in &page.figures {
            assert!(
                frame.width() >= 45.0,
                "figure reduite a {} mm",
                frame.width()
            );
            assert!((frame.height() / frame.width() - png_aspect()).abs() < 1e-4);
        }
    }

    #[test]
    fn puts_the_skull_anchor_at_the_top_of_the_figure() {
        let page = layout_for(&[]);
        let frame = frame_of(&page, "height");
        let y = callout(&page, "height").anchor.1;
        assert!(
            frame.top - y < frame.height() * 0.05,
            "l'ancre du crane doit etre dans les 5 % hauts, obtenu {y} pour un sommet a {}",
            frame.top
        );
    }

    #[test]
    fn puts_a_ground_anchor_at_the_bottom_of_the_figure() {
        let page = layout_for(&[]);
        let frame = page.figures[0];
        assert_eq!(anchor_page_y(0.0, &frame), frame.bottom);
    }

    #[test]
    fn keeps_the_anatomical_order_of_the_catalog() {
        let page = layout_for(&[]);
        let ys: Vec<f32> = ["height", "neck", "chest", "waist", "hips"]
            .iter()
            .map(|slug| callout(&page, slug).anchor.1)
            .collect();
        for pair in ys.windows(2) {
            assert!(
                pair[0] > pair[1],
                "ordre anatomique casse : {} puis {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn keeps_every_anchor_inside_its_figure() {
        let page = layout_for(&[]);
        let anchored = body_catalog::catalog()
            .iter()
            .filter(|m| m.is_key && m.anchor.is_some());
        for m in anchored {
            let frame = frame_of(&page, &m.slug);
            let (x, y) = callout(&page, &m.slug).anchor;
            assert!(
                (frame.left..=frame.right).contains(&x) && (frame.bottom..=frame.top).contains(&y),
                "ancre hors figure pour {}",
                m.slug
            );
        }
    }

    #[test]
    fn never_runs_a_leader_across_the_whole_body() {
        for m in body_catalog::catalog().iter().filter(|m| m.is_key) {
            let Some(anchor) = &m.anchor else { continue };
            let reaches = if anchor.label_side == "left" {
                anchor.x <= 55.0
            } else {
                anchor.x >= 45.0
            };
            assert!(
                reaches,
                "{} : ancre a {} % pour un libelle a {}",
                m.slug, anchor.x, anchor.label_side
            );
        }
    }

    #[test]
    fn labels_a_single_measurement_with_its_value() {
        let rows = vec![row("waist", Some(86.0))];
        let waist = body_catalog::find("waist").unwrap();
        assert_eq!(callout_name(waist), "Tour de taille");
        assert_eq!(callout_value(waist, &rows), "86,0 cm");
    }

    #[test]
    fn labels_a_pair_with_both_sides() {
        let rows = vec![
            row("bicep_left", Some(31.0)),
            row("bicep_right", Some(31.5)),
        ];
        let bicep = body_catalog::find("bicep_left").unwrap();
        assert_eq!(callout_name(bicep), "Tour de bras G/D");
        assert_eq!(callout_value(bicep, &rows), "31,0 / 31,5 cm");
    }

    #[test]
    fn shows_a_dash_for_a_measurement_never_taken() {
        let waist = body_catalog::find("waist").unwrap();
        assert_eq!(callout_value(waist, &[]), "- cm");
    }

    #[test]
    fn shows_a_dash_on_the_missing_side_of_a_pair() {
        let rows = vec![row("bicep_left", Some(31.0))];
        let bicep = body_catalog::find("bicep_left").unwrap();
        assert_eq!(callout_value(bicep, &rows), "31,0 / - cm");
    }
}
