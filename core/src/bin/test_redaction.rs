use lopdf::{Document, Object, ObjectId, Stream};
use masking_tool_core::masker::{apply_masks, MaskRect};
use std::env;
use std::path::Path;

#[derive(Clone, Copy)]
enum FillColor {
    None,
    Gray(f64),
    Rgb(f64, f64, f64),
    Cmyk(f64, f64, f64, f64),
}

impl FillColor {
    fn is_black(&self) -> bool {
        const EPS: f64 = 1e-6;
        match *self {
            FillColor::Gray(g) => g.abs() <= EPS,
            FillColor::Rgb(r, g, b) => r.abs() <= EPS && g.abs() <= EPS && b.abs() <= EPS,
            FillColor::Cmyk(c, m, y, k) => {
                c.abs() <= EPS && m.abs() <= EPS && y.abs() <= EPS && (k - 1.0).abs() <= EPS
            }
            FillColor::None => false,
        }
    }
}

fn get_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v as f64),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

fn mat_mul(m1: [f64; 6], m2: [f64; 6]) -> [f64; 6] {
    let a = m1[0];
    let b = m1[1];
    let c = m1[2];
    let d = m1[3];
    let e = m1[4];
    let f = m1[5];

    let ma = m2[0];
    let mb = m2[1];
    let mc = m2[2];
    let md = m2[3];
    let me = m2[4];
    let mf = m2[5];

    [
        a * ma + b * mc,
        a * mb + b * md,
        c * ma + d * mc,
        c * mb + d * md,
        e * ma + f * mc + me,
        e * mb + f * md + mf,
    ]
}

fn apply_matrix(m: [f64; 6], x: f64, y: f64) -> (f64, f64) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}

fn update_bbox(bbox: &mut Option<(f64, f64, f64, f64)>, x: f64, y: f64) {
    if let Some((min_x, min_y, max_x, max_y)) = bbox {
        *min_x = min_x.min(x);
        *min_y = min_y.min(y);
        *max_x = max_x.max(x);
        *max_y = max_y.max(y);
    } else {
        *bbox = Some((x, y, x, y));
    }
}

fn decode_stream_content(stream: &Stream) -> Option<Vec<u8>> {
    if stream.dict.get(b"Filter").is_err() {
        return Some(stream.content.clone());
    }
    stream.decompressed_content().ok()
}

fn walk_ops(
    doc: &Document,
    ops: Vec<lopdf::content::Operation>,
    resources: &lopdf::Dictionary,
    ctm_in: [f64; 6],
    text_ops: &mut usize,
    black_shapes: &mut Vec<(f64, f64, f64, f64)>,
) {
    let mut ctm = ctm_in;
    let mut stack: Vec<([f64; 6], FillColor)> = Vec::new();
    let mut fill = FillColor::None;
    let mut path_bbox: Option<(f64, f64, f64, f64)> = None;

    for op in ops {
        match op.operator.as_str() {
            "q" => stack.push((ctm, fill)),
            "Q" => {
                if let Some((prev_ctm, prev_fill)) = stack.pop() {
                    ctm = prev_ctm;
                    fill = prev_fill;
                }
                path_bbox = None;
            }
            "cm" => {
                if op.operands.len() == 6 {
                    let mut m = [0.0; 6];
                    for i in 0..6 {
                        m[i] = get_f64(&op.operands[i]).unwrap_or(0.0);
                    }
                    // 修正: cmは新しい行列を左から掛ける (M_cm * CTM_old)
                    ctm = mat_mul(m, ctm);
                }
            }
            "rg" => {
                if op.operands.len() >= 3 {
                    let r = get_f64(&op.operands[0]).unwrap_or(0.0);
                    let g = get_f64(&op.operands[1]).unwrap_or(0.0);
                    let b = get_f64(&op.operands[2]).unwrap_or(0.0);
                    fill = FillColor::Rgb(r, g, b);
                }
            }
            "g" => {
                if let Some(v) = op.operands.get(0).and_then(get_f64) {
                    fill = FillColor::Gray(v);
                }
            }
            "k" => {
                if op.operands.len() >= 4 {
                    let c = get_f64(&op.operands[0]).unwrap_or(0.0);
                    let m = get_f64(&op.operands[1]).unwrap_or(0.0);
                    let y = get_f64(&op.operands[2]).unwrap_or(0.0);
                    let k = get_f64(&op.operands[3]).unwrap_or(0.0);
                    fill = FillColor::Cmyk(c, m, y, k);
                }
            }
            "m" => {
                if op.operands.len() >= 2 {
                    let x = get_f64(&op.operands[0]).unwrap_or(0.0);
                    let y = get_f64(&op.operands[1]).unwrap_or(0.0);
                    let (tx, ty) = apply_matrix(ctm, x, y);
                    update_bbox(&mut path_bbox, tx, ty);
                }
            }
            "l" => {
                if op.operands.len() >= 2 {
                    let x = get_f64(&op.operands[0]).unwrap_or(0.0);
                    let y = get_f64(&op.operands[1]).unwrap_or(0.0);
                    let (tx, ty) = apply_matrix(ctm, x, y);
                    update_bbox(&mut path_bbox, tx, ty);
                }
            }
            "c" => {
                if op.operands.len() >= 6 {
                    for i in (0..6).step_by(2) {
                        let x = get_f64(&op.operands[i]).unwrap_or(0.0);
                        let y = get_f64(&op.operands[i + 1]).unwrap_or(0.0);
                        let (tx, ty) = apply_matrix(ctm, x, y);
                        update_bbox(&mut path_bbox, tx, ty);
                    }
                }
            }
            "v" => {
                if op.operands.len() >= 4 {
                    for i in (0..4).step_by(2) {
                        let x = get_f64(&op.operands[i]).unwrap_or(0.0);
                        let y = get_f64(&op.operands[i + 1]).unwrap_or(0.0);
                        let (tx, ty) = apply_matrix(ctm, x, y);
                        update_bbox(&mut path_bbox, tx, ty);
                    }
                }
            }
            "y" => {
                if op.operands.len() >= 4 {
                    for i in (0..4).step_by(2) {
                        let x = get_f64(&op.operands[i]).unwrap_or(0.0);
                        let y = get_f64(&op.operands[i + 1]).unwrap_or(0.0);
                        let (tx, ty) = apply_matrix(ctm, x, y);
                        update_bbox(&mut path_bbox, tx, ty);
                    }
                }
            }
            "re" => {
                if op.operands.len() >= 4 {
                    let x = get_f64(&op.operands[0]).unwrap_or(0.0);
                    let y = get_f64(&op.operands[1]).unwrap_or(0.0);
                    let w = get_f64(&op.operands[2]).unwrap_or(0.0);
                    let h = get_f64(&op.operands[3]).unwrap_or(0.0);
                    let corners = [
                        (x, y),
                        (x + w, y),
                        (x, y + h),
                        (x + w, y + h),
                    ];
                    for (cx, cy) in corners {
                        let (tx, ty) = apply_matrix(ctm, cx, cy);
                        update_bbox(&mut path_bbox, tx, ty);
                    }
                }
            }
            "Tj" | "TJ" | "'" | "\"" => {
                *text_ops += 1;
            }
            "Do" => {
                let name = op
                    .operands
                    .get(0)
                    .and_then(|o| o.as_name().ok())
                    .map(|v| v.to_vec());
                let Some(name) = name else { continue };
                let xobj_dict = match resources.get(b"XObject").and_then(|o| o.as_dict()) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let xobj = match xobj_dict.get(&name) {
                    Ok(o) => o,
                    Err(_) => continue,
                };
                let stream = match xobj {
                    Object::Reference(id) => doc.get_object(*id).ok().and_then(|o| o.as_stream().ok()),
                    Object::Stream(s) => Some(s),
                    _ => None,
                };
                let Some(stream) = stream else { continue };
                let subtype = stream
                    .dict
                    .get(b"Subtype")
                    .and_then(|o| o.as_name_str())
                    .unwrap_or("");
                if subtype != "Form" {
                    continue;
                }
                let form_matrix = stream
                    .dict
                    .get(b"Matrix")
                    .and_then(|o| o.as_array())
                    .ok()
                    .map(|arr| {
                        let mut m = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                        if arr.len() == 6 {
                            for i in 0..6 {
                                m[i] = get_f64(&arr[i]).unwrap_or(m[i]);
                            }
                        }
                        m
                    })
                    .unwrap_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

                let form_ctm = mat_mul(form_matrix, ctm);
                let form_resources = stream
                    .dict
                    .get(b"Resources")
                    .and_then(|o| o.as_dict())
                    .ok()
                    .cloned()
                    .unwrap_or_else(lopdf::Dictionary::new);
                let data = match decode_stream_content(stream) {
                    Some(d) => d,
                    None => continue,
                };
                let content = match lopdf::content::Content::decode(&data) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                walk_ops(doc, content.operations, &form_resources, form_ctm, text_ops, black_shapes);
            }
            "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" => {
                if let Some((min_x, min_y, max_x, max_y)) = path_bbox.take() {
                    if fill.is_black() {
                        black_shapes.push((min_x, min_y, max_x - min_x, max_y - min_y));
                    }
                }
            }
            "S" | "s" | "n" => {
                path_bbox = None;
            }
            _ => {}
        }
    }
}

fn get_media_box(doc: &Document, page_id: ObjectId) -> Option<[f64; 4]> {
    let page = doc.get_object(page_id).ok()?.as_dict().ok()?;
    let mb = page.get(b"MediaBox").ok()?.as_array().ok()?;
    if mb.len() == 4 {
        Some([
            get_f64(&mb[0]).unwrap_or(0.0),
            get_f64(&mb[1]).unwrap_or(0.0),
            get_f64(&mb[2]).unwrap_or(0.0),
            get_f64(&mb[3]).unwrap_or(0.0),
        ])
    } else {
        None
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: test_redaction <input.pdf> [output.pdf]");
        std::process::exit(1);
    }

    let input = &args[1];
    let output = args.get(2).cloned().unwrap_or_else(|| {
        let path = Path::new(input);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}_masked.pdf", stem)).to_string_lossy().to_string()
    });

    let mut doc = Document::load(input)?;
    let pages = doc.get_pages();
    println!("Pages: {}", pages.len());

    if let Some(page_id) = pages.get(&1) {
        if let Some(mb) = get_media_box(&doc, *page_id) {
            println!("Page1 MediaBox: {:?}", mb);
        }

        if let Ok(text) = doc.extract_text(&[1]) {
            let leaked: Vec<&str> = text.lines().filter(|l| l.contains("株式会社")).collect();
            println!("Leaked lines (original):");
            if leaked.is_empty() {
                println!("  (none)");
            } else {
                for line in leaked.iter().take(10) {
                    println!("  {}", line);
                }
            }
        }

        let mut text_ops = 0usize;
        let mut black_shapes = Vec::new();
        let resources = doc
            .get_object(*page_id)
            .and_then(|p| p.as_dict())
            .and_then(|d| d.get(b"Resources"))
            .and_then(|o| o.as_dict())
            .ok()
            .cloned()
            .unwrap_or_else(lopdf::Dictionary::new);
        if let Ok(content) = doc.get_and_decode_page_content(*page_id) {
            walk_ops(
                &doc,
                content.operations,
                &resources,
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                &mut text_ops,
                &mut black_shapes,
            );
        }

        println!("Text show operations found (page+forms): {}", text_ops);
        println!("Black filled shapes found (page+forms): {}", black_shapes.len());
        for (i, r) in black_shapes.iter().enumerate().take(10) {
            println!(
                "  {}: x={:.2}, y={:.2}, w={:.2}, h={:.2}",
                i + 1,
                r.0,
                r.1,
                r.2,
                r.3
            );
        }

        let page_box = get_media_box(&doc, *page_id).unwrap_or([0.0, 0.0, 595.0, 842.0]);
        let (px0, py0, px1, py1) = (page_box[0], page_box[1], page_box[2], page_box[3]);
        let mut candidate = None;
        for r in black_shapes.iter() {
            if r.2.abs() <= 1.0 || r.3.abs() <= 1.0 {
                continue;
            }
            let rx0 = r.0;
            let ry0 = r.1;
            let rx1 = r.0 + r.2;
            let ry1 = r.1 + r.3;
            let intersects = rx0 <= px1 && rx1 >= px0 && ry0 <= py1 && ry1 >= py0;
            if !intersects {
                continue;
            }
            let top = ry1;
            candidate = match candidate {
                None => Some((*r, top)),
                Some((prev, prev_top)) => {
                    if top > prev_top {
                        Some((*r, top))
                    } else {
                        Some((prev, prev_top))
                    }
                }
            };
        }

        if let Some((best, _)) = candidate {
            let (x, y, w, h) = best;
            let masks = vec![MaskRect {
                page: 1,
                x,
                y,
                width: w,
                height: h,
                color: "black".to_string(),
            }];

            apply_masks(&mut doc, &masks)?;
            let mut buffer = Vec::new();
            doc.save_to(&mut buffer)?;
            std::fs::write(&output, buffer)?;
            println!("Saved masked PDF: {}", output);

            let masked_doc = Document::load(&output)?;
            if let Ok(text) = masked_doc.extract_text(&[1]) {
                let leaked: Vec<&str> = text.lines().filter(|l| l.contains("株式会社")).collect();
                println!("Leaked lines (masked):");
                if leaked.is_empty() {
                    println!("  (none)");
                } else {
                    for line in leaked.iter().take(10) {
                        println!("  {}", line);
                    }
                }
            }
        } else {
            println!("No suitable black rectangle found to use as mask.");
        }
    }

    Ok(())
}
