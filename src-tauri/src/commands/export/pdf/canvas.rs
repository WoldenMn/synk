use crate::error::AppError;
use printpdf::{
    BuiltinFont, Color, ImageOptimizationOptions, Line, LinePoint, Mm, Op, PaintMode, PdfDocument,
    PdfFontHandle, PdfPage, PdfParseErrorSeverity, PdfSaveOptions, PdfWarnMsg, Point, Polygon,
    PolygonRing, Pt, RawImage, TextItem, WindingOrder, XObjectTransform,
};
use std::path::Path;

use super::layout::{PAGE_HEIGHT_MM, PAGE_WIDTH_MM};

const DISC_SIDES: usize = 16;

pub(crate) struct Canvas {
    doc: PdfDocument,
    pages: Vec<Vec<Op>>,
}

impl Canvas {
    pub(crate) fn new(title: &str) -> Self {
        Self {
            doc: PdfDocument::new(title),
            pages: vec![Vec::new()],
        }
    }

    pub(crate) fn add_page(&mut self) {
        self.pages.push(Vec::new());
    }

    fn ops(&mut self) -> &mut Vec<Op> {
        self.pages.last_mut().expect("une page au moins")
    }

    pub(crate) fn use_text(
        &mut self,
        text: impl Into<String>,
        size: f32,
        x: Mm,
        y: Mm,
        font: BuiltinFont,
    ) {
        self.ops().extend([
            Op::StartTextSection,
            Op::SetFont {
                font: PdfFontHandle::Builtin(font),
                size: Pt(size),
            },
            Op::SetTextCursor {
                pos: Point::new(x, y),
            },
            Op::ShowText {
                items: vec![TextItem::Text(text.into())],
            },
            Op::EndTextSection,
        ]);
    }

    pub(crate) fn set_outline(&mut self, thickness: f32, color: Color) {
        self.ops().extend([
            Op::SetOutlineThickness { pt: Pt(thickness) },
            Op::SetOutlineColor { col: color },
        ]);
    }

    pub(crate) fn add_polyline(&mut self, points_mm: &[(f32, f32)]) {
        let points = points_mm
            .iter()
            .map(|&(x, y)| LinePoint {
                p: Point::new(Mm(x), Mm(y)),
                bezier: false,
            })
            .collect();
        self.ops().push(Op::DrawLine {
            line: Line {
                points,
                is_closed: false,
            },
        });
    }

    pub(crate) fn fill_discs(&mut self, centers_mm: &[(f32, f32)], radius_mm: f32, color: Color) {
        let rings = centers_mm
            .iter()
            .map(|&(cx, cy)| PolygonRing {
                points: (0..DISC_SIDES)
                    .map(|k| {
                        let a = k as f32 / DISC_SIDES as f32 * std::f32::consts::TAU;
                        LinePoint {
                            p: Point::new(
                                Mm(cx + radius_mm * a.cos()),
                                Mm(cy + radius_mm * a.sin()),
                            ),
                            bezier: false,
                        }
                    })
                    .collect(),
            })
            .collect();
        self.ops().extend([
            Op::SaveGraphicsState,
            Op::SetFillColor { col: color },
            Op::DrawPolygon {
                polygon: Polygon {
                    rings,
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::NonZero,
                },
            },
            Op::RestoreGraphicsState,
        ]);
    }

    pub(crate) fn decode_image(bytes: &[u8]) -> Result<RawImage, AppError> {
        let mut warnings = Vec::new();
        let image =
            RawImage::decode_from_bytes(bytes, &mut warnings).map_err(AppError::Internal)?;
        fail_on_errors(&warnings)?;
        Ok(image)
    }

    pub(crate) fn place_image(&mut self, image: &RawImage, transform: XObjectTransform) {
        let id = self.doc.add_image(image);
        self.ops().push(Op::UseXobject { id, transform });
    }

    pub(crate) fn save(mut self, output_path: &Path) -> Result<(), AppError> {
        let pages = std::mem::take(&mut self.pages)
            .into_iter()
            .map(|ops| PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops))
            .collect();
        let options = PdfSaveOptions {
            optimize: false,
            image_optimization: Some(ImageOptimizationOptions {
                max_image_size: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        let bytes = self.doc.with_pages(pages).save(&options, &mut warnings);
        fail_on_errors(&warnings)?;
        std::fs::write(output_path, bytes)
            .map_err(|e| AppError::Internal(format!("écriture de {} : {e}", output_path.display())))
    }
}

fn fail_on_errors(warnings: &[PdfWarnMsg]) -> Result<(), AppError> {
    for w in warnings {
        match w.severity {
            PdfParseErrorSeverity::Error => {}
            PdfParseErrorSeverity::Warning => {
                log::warn!("printpdf, page {} op {} : {}", w.page, w.op_id, w.msg);
            }
            PdfParseErrorSeverity::Info => {
                log::debug!("printpdf, page {} op {} : {}", w.page, w.op_id, w.msg);
            }
        }
    }
    let errors: Vec<&str> = warnings
        .iter()
        .filter(|w| w.severity == PdfParseErrorSeverity::Error)
        .map(|w| w.msg.as_str())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Internal(format!(
            "PDF invalide : {}",
            errors.join(" ; ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::{RawImageData, RawImageFormat};

    fn colour_image(width: usize, height: usize) -> RawImage {
        RawImage {
            pixels: RawImageData::U8([10, 100, 200].repeat(width * height)),
            width,
            height,
            data_format: RawImageFormat::RGB8,
            tag: Vec::new(),
        }
    }

    #[test]
    fn keeps_an_image_above_two_megabytes_at_its_original_size() {
        let mut canvas = Canvas::new("test");
        canvas.place_image(&colour_image(1000, 1000), XObjectTransform::default());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grande.pdf");
        canvas.save(&path).unwrap();

        let doc = lopdf::Document::load(&path).unwrap();
        let page = *doc.get_pages().get(&1).unwrap();
        let images = doc.get_page_images(page).unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!((images[0].width, images[0].height), (1000, 1000));
    }

    #[test]
    fn refuses_bytes_that_are_not_an_image() {
        assert!(Canvas::decode_image(b"pas une image").is_err());
    }
}
