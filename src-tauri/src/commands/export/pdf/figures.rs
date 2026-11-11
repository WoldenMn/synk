use crate::error::AppError;
use printpdf::RawImage;

use super::canvas::Canvas;

const FRONT_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/assets/body/front.png"
));
const BACK_PNG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../src/assets/body/back.png"
));

pub(crate) fn figures() -> Result<[RawImage; 2], AppError> {
    Ok([
        Canvas::decode_image(FRONT_PNG)?,
        Canvas::decode_image(BACK_PNG)?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use printpdf::RawImageFormat;

    #[test]
    fn decodes_both_figures_with_their_transparency() {
        let [front, back] = figures().unwrap();
        for figure in [&front, &back] {
            assert_eq!(figure.data_format, RawImageFormat::RG8);
            assert!(figure.width > 0 && figure.height > figure.width);
        }
    }

    #[test]
    fn gives_both_views_the_same_pixel_size() {
        let [front, back] = figures().unwrap();
        assert_eq!((front.width, front.height), (back.width, back.height));
    }
}
