pub fn pdf_hex(s: &str) -> String {
    s.chars()
        .map(|c| {
            let code = u32::from(c);
            assert!(code <= 0xFF, "caractere hors WinAnsi dans l'attendu : {c}");
            format!("{code:02X}")
        })
        .collect()
}

fn pdf_literal(s: &str) -> Option<String> {
    if !s.bytes().all(|b| (32..=126).contains(&b)) {
        return None;
    }
    let mut out = String::new();
    for c in s.chars() {
        if matches!(c, '(' | ')' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    Some(out)
}

pub fn pdf_has_text(pdf: &str, s: &str) -> bool {
    pdf.contains(&pdf_hex(s)) || pdf_literal(s).is_some_and(|lit| pdf.contains(&lit))
}

#[allow(dead_code)]
pub fn page_one_xobject_draws(bytes: &[u8]) -> usize {
    let doc = lopdf::Document::load_mem(bytes).expect("PDF lisible");
    let page = *doc.get_pages().get(&1).expect("page 1");
    let content =
        lopdf::content::Content::decode(&doc.get_page_content(page)).expect("flux de page lisible");
    content
        .operations
        .iter()
        .filter(|op| op.operator == "Do")
        .count()
}

pub struct PageImage {
    pub width: i64,
    pub height: i64,
    pub color_space: Option<String>,
    pub has_soft_mask: bool,
}

pub fn page_one_images(bytes: &[u8]) -> Vec<PageImage> {
    let doc = lopdf::Document::load_mem(bytes).expect("PDF lisible");
    let page = *doc.get_pages().get(&1).expect("page 1");
    doc.get_page_images(page)
        .expect("images de la page 1 lisibles")
        .into_iter()
        .map(|image| PageImage {
            width: image.width,
            height: image.height,
            color_space: image.color_space,
            has_soft_mask: image.origin_dict.has(b"SMask"),
        })
        .collect()
}

fn multiply(m: [f32; 6], n: [f32; 6]) -> [f32; 6] {
    [
        m[0] * n[0] + m[1] * n[2],
        m[0] * n[1] + m[1] * n[3],
        m[2] * n[0] + m[3] * n[2],
        m[2] * n[1] + m[3] * n[3],
        m[4] * n[0] + m[5] * n[2] + n[4],
        m[4] * n[1] + m[5] * n[3] + n[5],
    ]
}

pub fn page_one_draw_matrices(bytes: &[u8]) -> Vec<[f32; 6]> {
    const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let doc = lopdf::Document::load_mem(bytes).expect("PDF lisible");
    let page = *doc.get_pages().get(&1).expect("page 1");
    let content =
        lopdf::content::Content::decode(&doc.get_page_content(page)).expect("flux de page lisible");
    let mut current = IDENTITY;
    let mut saved = Vec::new();
    let mut draws = Vec::new();
    for op in &content.operations {
        match op.operator.as_str() {
            "q" => saved.push(current),
            "Q" => current = saved.pop().expect("Q sans q"),
            "cm" => {
                let numbers: Vec<f32> = op
                    .operands
                    .iter()
                    .map(|o| o.as_float().expect("operande numerique"))
                    .collect();
                let m: [f32; 6] = numbers.try_into().expect("six operandes pour cm");
                current = multiply(m, current);
            }
            "Do" => draws.push(current),
            _ => {}
        }
    }
    draws
}
