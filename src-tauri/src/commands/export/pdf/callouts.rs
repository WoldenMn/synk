use super::layout::PAGE_WIDTH_MM;
use super::text_metrics::TextMetrics;

pub(crate) const LABEL_PT: f32 = 7.0;
pub(crate) const NAME_RISE_MM: f32 = 0.9;
pub(crate) const VALUE_DROP_MM: f32 = 2.7;
pub(crate) const SIDE_MARGIN_MM: f32 = 8.0;
pub(crate) const AREA_BOTTOM_MM: f32 = 24.0;
pub(crate) const AREA_TOP_MM: f32 = 256.0;
const BOX_ABOVE_MM: f32 = 2.8;
const BOX_BELOW_MM: f32 = 3.3;
const PITCH_MM: f32 = 7.5;
const GUTTER_MM: f32 = 3.0;
const BETWEEN_VIEWS_MM: f32 = 5.0;
const MAX_FIGURE_WIDTH_MM: f32 = 70.0;
pub(crate) const VIEWS: [&str; 2] = ["front", "back"];

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Rect {
    pub(crate) left: f32,
    pub(crate) bottom: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
}

impl Rect {
    pub(crate) fn width(&self) -> f32 {
        self.right - self.left
    }

    pub(crate) fn height(&self) -> f32 {
        self.top - self.bottom
    }

    #[cfg(test)]
    pub(crate) fn overlaps(&self, other: &Rect) -> bool {
        self.left < other.right
            && other.left < self.right
            && self.bottom < other.top
            && other.bottom < self.top
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Right,
}

pub(crate) struct Label {
    pub(crate) view: String,
    pub(crate) side: Side,
    pub(crate) x_pct: f32,
    pub(crate) y_pct: f32,
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) struct Callout {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) anchor: (f32, f32),
    pub(crate) leader: [(f32, f32); 4],
    pub(crate) text_x: f32,
    pub(crate) line_y: f32,
    pub(crate) text_box: Rect,
}

pub(crate) struct SheetLayout {
    pub(crate) figures: [Rect; 2],
    pub(crate) callouts: Vec<Callout>,
}

pub(crate) fn anchor_page_y(anchor_y: f32, frame: &Rect) -> f32 {
    frame.bottom + anchor_y / 100.0 * frame.height()
}

pub(crate) fn spread(desired: &[f32], pitch: f32, low: f32, high: f32) -> Vec<f32> {
    struct Block {
        len: usize,
        sum: f32,
    }
    let top = |b: &Block| b.sum / b.len as f32 + (b.len - 1) as f32 * pitch / 2.0;
    let mut blocks: Vec<Block> = Vec::new();
    for &y in desired {
        blocks.push(Block { len: 1, sum: y });
        while let [.., upper, lower] = blocks.as_slice() {
            let upper_bottom = top(upper) - (upper.len - 1) as f32 * pitch;
            if upper_bottom - top(lower) >= pitch {
                break;
            }
            let lower = blocks.pop().expect("deux blocs au moins");
            let upper = blocks.last_mut().expect("deux blocs au moins");
            upper.len += lower.len;
            upper.sum += lower.sum;
        }
    }
    let mut out: Vec<f32> = blocks
        .iter()
        .flat_map(|b| (0..b.len).map(move |k| top(b) - k as f32 * pitch))
        .collect();
    for i in 0..out.len() {
        let cap = if i == 0 { high } else { out[i - 1] - pitch };
        out[i] = out[i].min(cap);
    }
    for i in (0..out.len()).rev() {
        let floor = if i + 1 == out.len() {
            low
        } else {
            out[i + 1] + pitch
        };
        out[i] = out[i].max(floor);
    }
    out
}

fn with_gutter(width: f32) -> f32 {
    if width > 0.0 {
        width + GUTTER_MM
    } else {
        0.0
    }
}

pub(crate) fn sheet_layout(labels: &[Label], aspect: f32, metrics: &TextMetrics) -> SheetLayout {
    let text_width = |l: &Label| {
        metrics
            .width_mm(&l.name, LABEL_PT, false)
            .max(metrics.width_mm(&l.value, LABEL_PT, true))
    };
    let column_width = |view: &str, side: Side| {
        labels
            .iter()
            .filter(|l| l.view == view && l.side == side)
            .map(text_width)
            .fold(0.0f32, f32::max)
    };
    let widths = VIEWS.map(|v| [column_width(v, Side::Left), column_width(v, Side::Right)]);
    let fixed = widths
        .iter()
        .map(|[l, r]| with_gutter(*l) + with_gutter(*r))
        .sum::<f32>()
        + BETWEEN_VIEWS_MM;
    let room = PAGE_WIDTH_MM - 2.0 * SIDE_MARGIN_MM - fixed;
    let figure_width = (room / 2.0)
        .min(MAX_FIGURE_WIDTH_MM)
        .min((AREA_TOP_MM - AREA_BOTTOM_MM) / aspect);
    let figure_height = figure_width * aspect;
    let bottom = AREA_BOTTOM_MM + (AREA_TOP_MM - AREA_BOTTOM_MM - figure_height) / 2.0;

    let mut x = (PAGE_WIDTH_MM - fixed - 2.0 * figure_width) / 2.0;
    let mut figures = [Rect::default(); 2];
    let mut columns: Vec<(usize, Side, f32, f32)> = Vec::new();
    for (i, [left_w, right_w]) in widths.into_iter().enumerate() {
        columns.push((i, Side::Left, x, x + left_w));
        x += with_gutter(left_w);
        figures[i] = Rect {
            left: x,
            bottom,
            right: x + figure_width,
            top: bottom + figure_height,
        };
        x += figure_width;
        let right_start = x + with_gutter(right_w) - right_w;
        columns.push((i, Side::Right, right_start, right_start + right_w));
        x += with_gutter(right_w) + BETWEEN_VIEWS_MM;
    }

    let mut callouts = Vec::new();
    for (i, side, col_left, col_right) in columns {
        let frame = figures[i];
        let mut group: Vec<&Label> = labels
            .iter()
            .filter(|l| l.view == VIEWS[i] && l.side == side)
            .collect();
        group.sort_by(|a, b| b.y_pct.total_cmp(&a.y_pct));
        let desired: Vec<f32> = group
            .iter()
            .map(|l| anchor_page_y(l.y_pct, &frame))
            .collect();
        let lines = spread(
            &desired,
            PITCH_MM,
            AREA_BOTTOM_MM + BOX_BELOW_MM,
            AREA_TOP_MM - BOX_ABOVE_MM,
        );
        for ((label, anchor_y), line_y) in group.into_iter().zip(desired).zip(lines) {
            let anchor = (frame.left + label.x_pct / 100.0 * frame.width(), anchor_y);
            let (edge, near, far) = match side {
                Side::Left => (frame.left, col_right, col_left),
                Side::Right => (frame.right, col_left, col_right),
            };
            callouts.push(Callout {
                name: label.name.clone(),
                value: label.value.clone(),
                anchor,
                leader: [anchor, (edge, anchor_y), (near, line_y), (far, line_y)],
                text_x: col_left,
                line_y,
                text_box: Rect {
                    left: col_left,
                    bottom: line_y - BOX_BELOW_MM,
                    right: col_left + text_width(label),
                    top: line_y + BOX_ABOVE_MM,
                },
            });
        }
    }
    SheetLayout { figures, callouts }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(slug: &str, view: &str, side: Side, x_pct: f32, y_pct: f32) -> Label {
        Label {
            view: view.to_string(),
            side,
            x_pct,
            y_pct,
            name: format!("Mesure {slug}"),
            value: "123,4 / 123,4 cm".to_string(),
        }
    }

    #[test]
    fn leaves_distant_lines_at_their_anchor() {
        assert_eq!(
            spread(&[200.0, 150.0, 100.0], 7.0, 0.0, 300.0),
            vec![200.0, 150.0, 100.0]
        );
    }

    #[test]
    fn centres_two_crowded_lines_on_their_anchors() {
        let lines = spread(&[100.0, 98.0], 7.0, 0.0, 300.0);
        assert_eq!(lines, vec![102.5, 95.5]);
    }

    #[test]
    fn keeps_the_order_the_pitch_and_the_bounds() {
        let lines = spread(&[251.0, 250.0, 249.0, 120.0], 7.0, 20.0, 252.0);
        assert!(lines[0] <= 252.0, "{lines:?}");
        for pair in lines.windows(2) {
            assert!(pair[0] - pair[1] >= 7.0 - 1e-4, "{lines:?}");
        }
        assert_eq!(lines[3], 120.0);
    }

    #[test]
    fn pushes_a_line_up_from_the_lower_bound() {
        assert_eq!(spread(&[10.0], 7.0, 20.0, 300.0), vec![20.0]);
    }

    #[test]
    fn keeps_every_label_beside_the_figures_and_apart() {
        let labels = vec![
            label("a", "front", Side::Left, 50.0, 70.0),
            label("b", "front", Side::Left, 15.0, 69.0),
            label("c", "front", Side::Right, 85.0, 60.0),
            label("d", "back", Side::Right, 50.0, 65.0),
            label("e", "back", Side::Right, 50.0, 64.0),
        ];
        let metrics = TextMetrics::helvetica().unwrap();
        let page = sheet_layout(&labels, 1400.0 / 512.0, &metrics);
        assert_eq!(page.callouts.len(), labels.len());
        for (i, c) in page.callouts.iter().enumerate() {
            for frame in &page.figures {
                assert!(!c.text_box.overlaps(frame), "{} touche une figure", c.name);
            }
            for other in &page.callouts[i + 1..] {
                assert!(
                    !c.text_box.overlaps(&other.text_box),
                    "{} et {}",
                    c.name,
                    other.name
                );
            }
            assert_eq!(c.leader[0], c.anchor);
            assert_eq!(c.leader[3].1, c.line_y);
            assert!(c.text_box.left >= SIDE_MARGIN_MM - 1e-3);
            assert!(c.text_box.right <= PAGE_WIDTH_MM - SIDE_MARGIN_MM + 1e-3);
        }
    }

    #[test]
    fn gives_both_figures_the_same_size_and_proportions() {
        let labels = vec![label("a", "front", Side::Left, 50.0, 50.0)];
        let metrics = TextMetrics::helvetica().unwrap();
        let aspect = 1400.0 / 512.0;
        let page = sheet_layout(&labels, aspect, &metrics);
        let [front, back] = page.figures;
        assert_eq!(front.width(), back.width());
        assert!((front.height() / front.width() - aspect).abs() < 1e-4);
        assert!(front.right < back.left);
        assert!(front.top <= AREA_TOP_MM && front.bottom >= AREA_BOTTOM_MM);
    }
}
