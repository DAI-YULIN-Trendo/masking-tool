use lopdf::content::{Content, Operation};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct MaskRect {
    pub page: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaskColor {
    Black,
    White,
}

impl MaskColor {
    fn from_str(s: &str) -> Self {
        match s {
            "white" | "White" | "WHITE" => MaskColor::White,
            _ => MaskColor::Black,
        }
    }
}

#[derive(Clone, Debug)]
struct Mask {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    color: MaskColor,
}

impl Mask {
    fn new(rect: &MaskRect) -> Self {
        let x1 = rect.x.min(rect.x + rect.width);
        let x2 = rect.x.max(rect.x + rect.width);
        let y1 = rect.y.min(rect.y + rect.height);
        let y2 = rect.y.max(rect.y + rect.height);
        Self {
            x1,
            y1,
            x2,
            y2,
            color: MaskColor::from_str(&rect.color),
        }
    }

    fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    fn intersects_rect(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> bool {
        self.x1 <= max_x && self.x2 >= min_x && self.y1 <= max_y && self.y2 >= min_y
    }
}

#[derive(Clone)]
struct MaskSet {
    masks: Vec<Mask>,
}

impl MaskSet {
    fn new(rects: &[MaskRect]) -> Self {
        let masks = rects.iter().map(Mask::new).collect();
        Self { masks }
    }

    fn intersects_rect(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> bool {
        self.masks
            .iter()
            .any(|m| m.intersects_rect(min_x, min_y, max_x, max_y))
    }

    fn color_at(&self, x: f64, y: f64) -> Option<MaskColor> {
        let mut white = false;
        for mask in &self.masks {
            if mask.contains_point(x, y) {
                if mask.color == MaskColor::Black {
                    return Some(MaskColor::Black);
                }
                white = true;
            }
        }
        if white {
            Some(MaskColor::White)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct TextState {
    text_matrix: [f64; 6],
    line_matrix: [f64; 6],
    font_size: f64,
    h_scale: f64,
    char_space: f64,
    word_space: f64,
    leading: f64,
    text_rise: f64,
    font_name: Option<Vec<u8>>,
}

impl TextState {
    fn new() -> Self {
        Self {
            text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            line_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            font_size: 0.0,
            h_scale: 1.0,
            char_space: 0.0,
            word_space: 0.0,
            leading: 0.0,
            text_rise: 0.0,
            font_name: None,
        }
    }

    fn reset_text_matrices(&mut self) {
        self.text_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        self.line_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    }
}

#[derive(Clone)]
struct GraphicsState {
    ctm: [f64; 6],
    text: TextState,
}

impl GraphicsState {
    fn new() -> Self {
        Self {
            ctm: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            text: TextState::new(),
        }
    }
}

#[derive(Clone)]
enum FontMetrics {
    Simple {
        first: u16,
        widths: Vec<f64>,
        missing: f64,
    },
    Type0 {
        widths: HashMap<u16, f64>,
        dw: f64,
        identity: bool,
    },
}

impl FontMetrics {
    fn width(&self, code: u16) -> f64 {
        match self {
            FontMetrics::Simple {
                first,
                widths,
                missing,
            } => {
                if code < *first {
                    return *missing;
                }
                let idx = (code - *first) as usize;
                widths.get(idx).copied().unwrap_or(*missing)
            }
            FontMetrics::Type0 { widths, dw, .. } => widths.get(&code).copied().unwrap_or(*dw),
        }
    }

    fn is_identity(&self) -> bool {
        matches!(self, FontMetrics::Type0 { identity: true, .. })
    }

    fn codes(&self, bytes: &[u8]) -> Vec<u16> {
        match self {
            FontMetrics::Type0 { identity: true, .. } => {
                let mut out = Vec::with_capacity(bytes.len() / 2 + 1);
                let mut i = 0;
                while i + 1 < bytes.len() {
                    let code = ((bytes[i] as u16) << 8) | (bytes[i + 1] as u16);
                    out.push(code);
                    i += 2;
                }
                if i < bytes.len() {
                    out.push(bytes[i] as u16);
                }
                out
            }
            _ => bytes.iter().map(|b| *b as u16).collect(),
        }
    }
}

#[derive(Clone)]
struct ResourceState {
    dict: Dictionary,
    xobject_names: HashSet<Vec<u8>>,
    counter: u32,
}

impl ResourceState {
    fn new(dict: Dictionary) -> Self {
        let mut names = HashSet::new();
        if let Ok(xobj_dict) = dict.get(b"XObject").and_then(|o| o.as_dict()) {
            for (k, _) in xobj_dict.iter() {
                names.insert(k.clone());
            }
        }
        Self {
            dict,
            xobject_names: names,
            counter: 0,
        }
    }

    fn next_xobject_name(&mut self) -> Vec<u8> {
        loop {
            self.counter += 1;
            let name = format!("XMask{}", self.counter).into_bytes();
            if !self.xobject_names.contains(&name) {
                self.xobject_names.insert(name.clone());
                return name;
            }
        }
    }

    fn add_xobject(&mut self, obj_id: ObjectId) -> Vec<u8> {
        let name = self.next_xobject_name();
        if self.dict.get(b"XObject").is_err() {
            self.dict.set("XObject", Dictionary::new());
        }
        if let Ok(xobj) = self.dict.get_mut(b"XObject") {
            if let Ok(xdict) = xobj.as_dict_mut() {
                xdict.set(name.clone(), Object::Reference(obj_id));
            }
        }
        name
    }
}

fn deref_dict(doc: &Document, obj: &Object) -> Option<Dictionary> {
    match obj {
        Object::Dictionary(d) => Some(d.clone()),
        Object::Reference(id) => doc.get_dictionary(*id).ok().cloned(),
        _ => None,
    }
}

fn merge_resource_dict(doc: &Document, base: &mut Dictionary, overlay: &Dictionary) {
    for (k, v) in overlay.iter() {
        // Resource categories are dictionaries (possibly indirect). Child overrides parent.
        let key = k.clone();
        let over_dict = match deref_dict(doc, v) {
            Some(d) => d,
            None => {
                base.set(key, v.clone());
                continue;
            }
        };

        let mut merged = match base.get(&key).ok().and_then(|existing| deref_dict(doc, existing)) {
            Some(d) => d,
            None => {
                base.set(key, Object::Dictionary(over_dict));
                continue;
            }
        };

        for (sk, sv) in over_dict.iter() {
            merged.set(sk.clone(), sv.clone());
        }
        base.set(key, Object::Dictionary(merged));
    }
}

fn build_inherited_page_resources(doc: &Document, page_id: ObjectId) -> Dictionary {
    let (direct_opt, resource_ids) = doc.get_page_resources(page_id);
    let mut out = Dictionary::new();

    // resource_ids is collected from leaf->root; inherit as root then overlay child.
    for id in resource_ids.iter().rev() {
        if let Ok(d) = doc.get_dictionary(*id) {
            merge_resource_dict(doc, &mut out, d);
        }
    }

    if let Some(direct) = direct_opt {
        merge_resource_dict(doc, &mut out, direct);
    }

    out
}

fn get_f64(obj: &Object) -> Option<f64> {
    match obj {
        Object::Real(v) => Some(*v as f64),
        Object::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

fn get_i64(obj: &Object) -> Option<i64> {
    match obj {
        Object::Integer(v) => Some(*v),
        Object::Real(v) => Some(*v as i64),
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

fn update_path_bbox(bbox: &mut Option<(f64, f64, f64, f64)>, x: f64, y: f64) {
    if let Some((min_x, min_y, max_x, max_y)) = bbox {
        if x < *min_x {
            *min_x = x;
        }
        if y < *min_y {
            *min_y = y;
        }
        if x > *max_x {
            *max_x = x;
        }
        if y > *max_y {
            *max_y = y;
        }
    } else {
        *bbox = Some((x, y, x, y));
    }
}

fn push_op(op: Operation, new_ops: &mut Vec<Operation>, keep_flags: &mut Vec<bool>) {
    new_ops.push(op);
    keep_flags.push(true);
}

fn translate_matrix(m: &mut [f64; 6], tx: f64, ty: f64) {
    m[4] += tx * m[0] + ty * m[2];
    m[5] += tx * m[1] + ty * m[3];
}

fn invert_matrix(m: [f64; 6]) -> Option<[f64; 6]> {
    let det = m[0] * m[3] - m[1] * m[2];
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;
    let a = m[3] * inv_det;
    let b = -m[1] * inv_det;
    let c = -m[2] * inv_det;
    let d = m[0] * inv_det;
    let e = (m[2] * m[5] - m[3] * m[4]) * inv_det;
    let f = (m[1] * m[4] - m[0] * m[5]) * inv_det;
    Some([a, b, c, d, e, f])
}

fn decode_form_content(stream: &Stream) -> anyhow::Result<Vec<u8>> {
    if stream.dict.get(b"Filter").is_err() {
        return Ok(stream.content.clone());
    }
    stream
        .decompressed_content()
        .map_err(|_| anyhow::anyhow!("Unsupported form stream filters"))
}

fn parse_font_metrics(doc: &Document, font_obj: &Object) -> Option<FontMetrics> {
    let dict = font_obj.as_dict().ok()?;
    let subtype = dict
        .get(b"Subtype")
        .and_then(|o| o.as_name_str())
        .unwrap_or("");

    if subtype == "Type0" {
        let encoding_name = dict
            .get(b"Encoding")
            .and_then(|o| o.as_name_str())
            .ok();
        let identity = matches!(encoding_name, Some("Identity-H" | "Identity-V"));

        let descendant = dict.get(b"DescendantFonts").and_then(|o| o.as_array()).ok()?;
        let first_desc = descendant.get(0)?;
        let desc_obj = resolve_object(doc, first_desc)?;
        let desc_dict = desc_obj.as_dict().ok()?;

        let dw = desc_dict
            .get(b"DW")
            .ok()
            .and_then(get_f64)
            .unwrap_or(1000.0)
            / 1000.0;

        let mut widths = HashMap::new();
        if let Ok(w_arr) = desc_dict.get(b"W").and_then(|o| o.as_array()) {
            let mut i = 0;
            while i < w_arr.len() {
                let c1 = match get_i64(&w_arr[i]) {
                    Some(v) => v as u16,
                    None => break,
                };
                if i + 1 >= w_arr.len() {
                    break;
                }
                if let Ok(arr) = w_arr[i + 1].as_array() {
                    for (j, w) in arr.iter().enumerate() {
                        if let Some(val) = get_f64(w) {
                            widths.insert(c1 + j as u16, val / 1000.0);
                        }
                    }
                    i += 2;
                } else if i + 2 < w_arr.len() {
                    let c2 = match get_i64(&w_arr[i + 1]) {
                        Some(v) => v as u16,
                        None => {
                            i += 1;
                            continue;
                        }
                    };
                    let w = match get_f64(&w_arr[i + 2]) {
                        Some(v) => v / 1000.0,
                        None => {
                            i += 3;
                            continue;
                        }
                    };
                    for cid in c1..=c2 {
                        widths.insert(cid, w);
                    }
                    i += 3;
                } else {
                    break;
                }
            }
        }

        Some(FontMetrics::Type0 {
            widths,
            dw,
            identity,
        })
    } else {
        let first = dict
            .get(b"FirstChar")
            .ok()
            .and_then(get_i64)
            .unwrap_or(0) as u16;
        let widths = dict
            .get(b"Widths")
            .and_then(|o| o.as_array())
            .ok()
            .map(|arr| {
                arr.iter()
                    .filter_map(get_f64)
                    .map(|v| v / 1000.0)
                    .collect::<Vec<f64>>()
            })
            .unwrap_or_default();

        let missing = dict
            .get(b"MissingWidth")
            .ok()
            .and_then(get_f64)
            .or_else(|| {
                dict.get(b"FontDescriptor")
                    .and_then(|o| o.as_dict())
                    .ok()
                    .and_then(|d| d.get(b"MissingWidth").ok().and_then(get_f64))
            })
            .unwrap_or(600.0)
            / 1000.0;

        Some(FontMetrics::Simple {
            first,
            widths,
            missing,
        })
    }
}

fn resolve_object<'a>(doc: &'a Document, obj: &'a Object) -> Option<&'a Object> {
    match obj {
        Object::Reference(id) => doc.get_object(*id).ok(),
        _ => Some(obj),
    }
}

fn get_font_metrics(
    doc: &Document,
    resources: &Dictionary,
    name: &[u8],
    cache: &mut HashMap<Vec<u8>, FontMetrics>,
) -> Option<FontMetrics> {
    if let Some(metrics) = cache.get(name) {
        return Some(metrics.clone());
    }
    let fonts = resources.get(b"Font").and_then(|o| o.as_dict()).ok()?;
    let font_entry = fonts.get(name).ok()?;
    let font_obj = resolve_object(doc, font_entry)?;
    let metrics = parse_font_metrics(doc, font_obj)?;
    cache.insert(name.to_vec(), metrics.clone());
    Some(metrics)
}

fn glyph_advance_for_bytes(bytes: &[u8], font: Option<&FontMetrics>, state: &TextState) -> f64 {
    let char_space = state.char_space;
    let word_space = state.word_space;

    if let Some(metrics) = font {
        let codes = metrics.codes(bytes);
        let mut total = 0.0;
        for code in codes {
            total += metrics.width(code);
            total += char_space;
            if code == 32 {
                total += word_space;
            }
        }
        total
    } else {
        // Fallback: assume average 0.5 em width.
        let mut total = bytes.len() as f64 * 0.5;
        total += bytes.len() as f64 * char_space;
        if bytes.iter().any(|b| *b == 32) {
            let spaces = bytes.iter().filter(|b| **b == 32).count() as f64;
            total += spaces * word_space;
        }
        total
    }
}

fn advance_for_operand(op: &Object, font: Option<&FontMetrics>, state: &TextState) -> f64 {
    match op {
        Object::String(bytes, _) => glyph_advance_for_bytes(bytes, font, state),
        Object::Array(items) => {
            let mut total = 0.0;
            for item in items {
                match item {
                    Object::String(bytes, _) => {
                        total += glyph_advance_for_bytes(bytes, font, state);
                    }
                    Object::Integer(v) => {
                        total += -(*v as f64) / 1000.0;
                    }
                    Object::Real(v) => {
                        total += -(*v as f64) / 1000.0;
                    }
                    _ => {}
                }
            }
            total
        }
        _ => 0.0,
    }
}

fn text_bbox(state: &GraphicsState, advance_glyph: f64) -> Option<(f64, f64, f64, f64)> {
    if state.text.font_size.abs() < 1e-6 {
        let (x, y) = apply_matrix(state.ctm, state.text.text_matrix[4], state.text.text_matrix[5]);
        return Some((x, y, x, y));
    }

    let scale = [
        state.text.font_size * state.text.h_scale,
        0.0,
        0.0,
        state.text.font_size,
        0.0,
        state.text.text_rise,
    ];
    let text_to_page = mat_mul(mat_mul(scale, state.text.text_matrix), state.ctm);

    let points = [
        (0.0, 0.0),
        (advance_glyph, 0.0),
        (0.0, 1.0),
        (advance_glyph, 1.0),
    ];

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for (x, y) in points {
        let (tx, ty) = apply_matrix(text_to_page, x, y);
        min_x = min_x.min(tx);
        min_y = min_y.min(ty);
        max_x = max_x.max(tx);
        max_y = max_y.max(ty);
    }

    Some((min_x, min_y, max_x, max_y))
}

fn handle_text_show(
    op: &mut Operation,
    state: &mut GraphicsState,
    font: Option<&FontMetrics>,
    masks: &MaskSet,
) -> bool {
    let operand = match op.operator.as_str() {
        "Tj" | "'" | "\"" => op.operands.get(0),
        "TJ" => op.operands.get(0),
        _ => None,
    };

    let advance_glyph = operand
        .map(|o| advance_for_operand(o, font, &state.text))
        .unwrap_or(0.0);

    let mut redacted = false;
    if let Some((min_x, min_y, max_x, max_y)) = text_bbox(state, advance_glyph) {
        if masks.intersects_rect(min_x, min_y, max_x, max_y) {
            redacted = true;
            match op.operator.as_str() {
                "TJ" => {
                    op.operands = vec![Object::Array(vec![])];
                }
                "Tj" | "'" | "\"" => {
                    if let Some(Object::String(_, fmt)) = op.operands.get(0) {
                        op.operands[0] = Object::String(Vec::new(), *fmt);
                    } else {
                        op.operands = vec![Object::String(Vec::new(), lopdf::StringFormat::Literal)];
                    }
                }
                _ => {}
            }
        }
    }

    // Update text matrix to reflect text advance.
    let advance_user = advance_glyph * state.text.font_size * state.text.h_scale;
    if advance_user.abs() > 1e-9 {
        translate_matrix(&mut state.text.text_matrix, advance_user, 0.0);
    }

    redacted
}

fn text_newline(state: &mut TextState) {
    let ty = -state.leading;
    translate_matrix(&mut state.line_matrix, 0.0, ty);
    state.text_matrix = state.line_matrix;
}

#[derive(Clone)]
enum ImageColor {
    Gray,
    Rgb,
    Cmyk,
}

impl ImageColor {
    fn channels(&self) -> usize {
        match self {
            ImageColor::Gray => 1,
            ImageColor::Rgb => 3,
            ImageColor::Cmyk => 4,
        }
    }
}

struct ImageData {
    width: usize,
    height: usize,
    color: ImageColor,
    pixels: Vec<u8>,
    decode: Vec<(f64, f64)>,
}

fn parse_decode_array(dict: &Dictionary, channels: usize) -> Vec<(f64, f64)> {
    if let Ok(arr) = dict.get(b"Decode").and_then(|o| o.as_array()) {
        let mut out = Vec::with_capacity(channels);
        let mut i = 0;
        while i + 1 < arr.len() && out.len() < channels {
            let dmin = get_f64(&arr[i]).unwrap_or(0.0);
            let dmax = get_f64(&arr[i + 1]).unwrap_or(1.0);
            out.push((dmin, dmax));
            i += 2;
        }
        if out.len() == channels {
            return out;
        }
    }
    vec![(0.0, 1.0); channels]
}

fn decode_image_stream(stream: &Stream) -> anyhow::Result<ImageData> {
    let width = stream
        .dict
        .get(b"Width")
        .ok()
        .and_then(get_i64)
        .ok_or_else(|| anyhow::anyhow!("Missing image Width"))? as usize;
    let height = stream
        .dict
        .get(b"Height")
        .ok()
        .and_then(get_i64)
        .ok_or_else(|| anyhow::anyhow!("Missing image Height"))? as usize;
    let bpc = stream
        .dict
        .get(b"BitsPerComponent")
        .ok()
        .and_then(get_i64)
        .unwrap_or(8) as u8;
    if bpc != 8 {
        return Err(anyhow::anyhow!("Unsupported BitsPerComponent"));
    }

    let color_space = stream
        .dict
        .get(b"ColorSpace")
        .and_then(|o| o.as_name_str())
        .ok();

    let filters = stream.filters().unwrap_or_default();
    let filter = filters.get(0).map(|s| s.as_str()).unwrap_or("");

    if filters.len() > 1 {
        return Err(anyhow::anyhow!("Unsupported multiple image filters"));
    }

    match filter {
        "DCTDecode" => {
            let mut decoder = jpeg_decoder::Decoder::new(&stream.content[..]);
            let pixels = decoder
                .decode()
                .map_err(|_| anyhow::anyhow!("Failed to decode JPEG"))?;
            let info = decoder
                .info()
                .ok_or_else(|| anyhow::anyhow!("Missing JPEG info"))?;
            let color = match info.pixel_format {
                jpeg_decoder::PixelFormat::L8 => ImageColor::Gray,
                jpeg_decoder::PixelFormat::RGB24 => ImageColor::Rgb,
                jpeg_decoder::PixelFormat::CMYK32 => ImageColor::Cmyk,
                _ => return Err(anyhow::anyhow!("Unsupported JPEG pixel format")),
            };
            if let Some(cs) = color_space {
                match (cs, &color) {
                    ("DeviceGray", ImageColor::Gray)
                    | ("DeviceRGB", ImageColor::Rgb)
                    | ("DeviceCMYK", ImageColor::Cmyk) => {}
                    _ => return Err(anyhow::anyhow!("Unsupported image ColorSpace")),
                }
            }
            let decode = parse_decode_array(&stream.dict, color.channels());
            Ok(ImageData {
                width: info.width as usize,
                height: info.height as usize,
                color,
                pixels,
                decode,
            })
        }
        "FlateDecode" | "" => {
            let data = if filter.is_empty() {
                stream.content.clone()
            } else {
                use flate2::read::ZlibDecoder;
                use std::io::Read;
                let mut decoder = ZlibDecoder::new(&stream.content[..]);
                let mut out = Vec::new();
                decoder
                    .read_to_end(&mut out)
                    .map_err(|_| anyhow::anyhow!("Failed to decompress image"))?;
                out
            };

            let color = match color_space {
                Some("DeviceGray") => ImageColor::Gray,
                Some("DeviceRGB") => ImageColor::Rgb,
                Some("DeviceCMYK") => ImageColor::Cmyk,
                None => return Err(anyhow::anyhow!("Missing image ColorSpace")),
                _ => return Err(anyhow::anyhow!("Unsupported image ColorSpace")),
            };

            let mut raw = data;
            if let Ok(params) = stream.dict.get(b"DecodeParms").and_then(|o| o.as_dict()) {
                let predictor = params
                    .get(b"Predictor")
                    .ok()
                    .and_then(get_i64)
                    .unwrap_or(1);
                if (10..=15).contains(&predictor) {
                    let colors = params
                        .get(b"Colors")
                        .ok()
                        .and_then(get_i64)
                        .unwrap_or(color.channels() as i64) as usize;
                    let columns = params
                        .get(b"Columns")
                        .ok()
                        .and_then(get_i64)
                        .unwrap_or(width as i64) as usize;
                    let bits = params
                        .get(b"BitsPerComponent")
                        .ok()
                        .and_then(get_i64)
                        .unwrap_or(8) as usize;
                    if bits != 8 {
                        return Err(anyhow::anyhow!("Unsupported predictor bit depth"));
                    }
                    let bytes_per_pixel = colors * bits / 8;
                    raw = lopdf::filters::png::decode_frame(&raw, bytes_per_pixel, columns)
                        .map_err(|_| anyhow::anyhow!("Failed to decode predictor"))?;
                }
            }

            let expected_len = width * height * color.channels();
            if raw.len() < expected_len {
                return Err(anyhow::anyhow!("Image data too short"));
            }
            raw.truncate(expected_len);

            let decode = parse_decode_array(&stream.dict, color.channels());
            Ok(ImageData {
                width,
                height,
                color,
                pixels: raw,
                decode,
            })
        }
        _ => Err(anyhow::anyhow!("Unsupported image filter")),
    }
}

fn encoded_value_for_display(value: f64, decode_min: f64, decode_max: f64) -> u8 {
    let denom = decode_max - decode_min;
    if denom.abs() < 1e-12 {
        return if value < 0.5 { 0 } else { 255 };
    }
    let v = (value - decode_min) / denom;
    let v = v.clamp(0.0, 1.0);
    (v * 255.0).round() as u8
}

fn mask_pixel_value(color: MaskColor, img_color: &ImageColor, decode: &[(f64, f64)]) -> Vec<u8> {
    let channels = img_color.channels();
    let mut out = Vec::with_capacity(channels);
    match img_color {
        ImageColor::Gray => {
            let display = if color == MaskColor::Black { 0.0 } else { 1.0 };
            let (dmin, dmax) = decode.get(0).copied().unwrap_or((0.0, 1.0));
            out.push(encoded_value_for_display(display, dmin, dmax));
        }
        ImageColor::Rgb => {
            let display = if color == MaskColor::Black { 0.0 } else { 1.0 };
            for i in 0..3 {
                let (dmin, dmax) = decode.get(i).copied().unwrap_or((0.0, 1.0));
                out.push(encoded_value_for_display(display, dmin, dmax));
            }
        }
        ImageColor::Cmyk => {
            let display = if color == MaskColor::Black {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            for i in 0..4 {
                let (dmin, dmax) = decode.get(i).copied().unwrap_or((0.0, 1.0));
                out.push(encoded_value_for_display(display[i], dmin, dmax));
            }
        }
    }
    out
}

fn mask_bounds_in_pixels(
    mask: &Mask,
    inv_ctm: Option<[f64; 6]>,
    width: usize,
    height: usize,
) -> Option<(usize, usize, usize, usize)> {
    let inv = inv_ctm?;
    let corners = [
        (mask.x1, mask.y1),
        (mask.x1, mask.y2),
        (mask.x2, mask.y1),
        (mask.x2, mask.y2),
    ];
    let mut min_u = f64::MAX;
    let mut max_u = f64::MIN;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;

    for (x, y) in corners {
        let (u, v) = apply_matrix(inv, x, y);
        min_u = min_u.min(u);
        max_u = max_u.max(u);
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }

    let mut x0 = (min_u * width as f64).floor() as isize;
    let mut x1 = (max_u * width as f64).ceil() as isize;
    let mut y0 = ((1.0 - max_v) * height as f64).floor() as isize;
    let mut y1 = ((1.0 - min_v) * height as f64).ceil() as isize;

    if x1 <= 0 || y1 <= 0 || x0 >= width as isize || y0 >= height as isize {
        return None;
    }

    x0 = x0.clamp(0, width as isize);
    x1 = x1.clamp(0, width as isize);
    y0 = y0.clamp(0, height as isize);
    y1 = y1.clamp(0, height as isize);

    if x0 >= x1 || y0 >= y1 {
        return None;
    }

    Some((x0 as usize, y0 as usize, x1 as usize, y1 as usize))
}

fn redact_image_stream(stream: &Stream, ctm: [f64; 6], masks: &MaskSet) -> anyhow::Result<Stream> {
    let mut image = decode_image_stream(stream)?;
    let channels = image.color.channels();
    let inv_ctm = invert_matrix(ctm);

    // Process white masks first, then black masks to ensure black wins on overlaps.
    for color in [MaskColor::White, MaskColor::Black] {
        for mask in &masks.masks {
            if mask.color != color {
                continue;
            }
            let bounds = if let Some(inv) = inv_ctm {
                mask_bounds_in_pixels(mask, Some(inv), image.width, image.height)
            } else {
                Some((0, 0, image.width, image.height))
            };
            let Some((x0, y0, x1, y1)) = bounds else {
                continue;
            };
            let fill = mask_pixel_value(color, &image.color, &image.decode);
            for py in y0..y1 {
                let v = 1.0 - (py as f64 + 0.5) / image.height as f64;
                for px in x0..x1 {
                    let u = (px as f64 + 0.5) / image.width as f64;
                    let (ux, uy) = apply_matrix(ctm, u, v);
                    if mask.contains_point(ux, uy) {
                        let idx = (py * image.width + px) * channels;
                        for c in 0..channels {
                            image.pixels[idx + c] = fill[c];
                        }
                    }
                }
            }
        }
    }

    let mut dict = stream.dict.clone();
    dict.set("Width", image.width as i64);
    dict.set("Height", image.height as i64);
    dict.set("BitsPerComponent", 8i64);
    match image.color {
        ImageColor::Gray => dict.set("ColorSpace", "DeviceGray"),
        ImageColor::Rgb => dict.set("ColorSpace", "DeviceRGB"),
        ImageColor::Cmyk => dict.set("ColorSpace", "DeviceCMYK"),
    }
    dict.remove(b"Filter");
    dict.remove(b"DecodeParms");

    let mut new_stream = Stream::new(dict, image.pixels);
    new_stream
        .compress()
        .map_err(|_| anyhow::anyhow!("Failed to compress image"))?;
    Ok(new_stream)
}

struct ProcessResult {
    ops: Vec<Operation>,
    resources: Dictionary,
    modified: bool,
}

// Form XObject の Resources を解決する（間接参照対応 + 親リソース継承）
fn resolve_form_resources(
    doc: &Document,
    form_dict: &Dictionary,
    parent_resources: &Dictionary,
) -> Dictionary {
    // まず Form XObject 自体の Resources を取得
    let form_res = form_dict
        .get(b"Resources")
        .ok()
        .and_then(|obj| match obj {
            // 直接辞書の場合
            Object::Dictionary(d) => Some(d.clone()),
            // 間接参照の場合 → ドキュメントから解決
            Object::Reference(id) => doc.get_dictionary(*id).ok().cloned(),
            _ => None,
        });

    match form_res {
        Some(mut res) => {
            // Form 自身のリソースに不足しているカテゴリを親から継承
            for (key, value) in parent_resources.iter() {
                if res.get(key).is_err() {
                    res.set(key.clone(), value.clone());
                } else {
                    // カテゴリ内で不足している個別エントリを親から補完
                    if let (Some(child_cat), Some(parent_cat)) = (
                        deref_dict(doc, res.get(key).unwrap()),
                        deref_dict(doc, value),
                    ) {
                        let mut merged = child_cat.clone();
                        for (sk, sv) in parent_cat.iter() {
                            if merged.get(sk).is_err() {
                                merged.set(sk.clone(), sv.clone());
                            }
                        }
                        res.set(key.clone(), Object::Dictionary(merged));
                    }
                }
            }
            res
        }
        None => {
            // Resources が無い場合は親のリソースをそのまま継承
            parent_resources.clone()
        }
    }
}

fn process_ops(
    ops: Vec<Operation>,
    mut state: GraphicsState,
    mut resources: ResourceState,
    doc: &mut Document,
    masks: &MaskSet,
    recursion: &mut Vec<ObjectId>,
) -> anyhow::Result<ProcessResult> {
    let mut new_ops = Vec::with_capacity(ops.len());
    let mut keep_flags: Vec<bool> = Vec::with_capacity(ops.len());
    let mut stack: Vec<GraphicsState> = Vec::new();
    let mut font_cache: HashMap<Vec<u8>, FontMetrics> = HashMap::new();
    let mut modified = false;
    let mut path_indices: Vec<usize> = Vec::new();
    let mut path_bbox: Option<(f64, f64, f64, f64)> = None;

    for mut op in ops {
        match op.operator.as_str() {
            "q" => {
                stack.push(state.clone());
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Q" => {
                if let Some(prev) = stack.pop() {
                    state = prev;
                }
                path_indices.clear();
                path_bbox = None;
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "cm" => {
                if op.operands.len() == 6 {
                    let mut m = [0.0; 6];
                    for i in 0..6 {
                        m[i] = get_f64(&op.operands[i]).unwrap_or(0.0);
                    }
                    // PDF仕様: CTM_new = M_cm × CTM_old
                    // cm行列を先に適用し、その上に既存CTMを適用する
                    state.ctm = mat_mul(m, state.ctm);
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "BT" => {
                state.text.reset_text_matrices();
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "ET" => {
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Tm" => {
                if op.operands.len() == 6 {
                    let mut m = [0.0; 6];
                    for i in 0..6 {
                        m[i] = get_f64(&op.operands[i]).unwrap_or(0.0);
                    }
                    state.text.text_matrix = m;
                    state.text.line_matrix = m;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Td" | "TD" => {
                if op.operands.len() == 2 {
                    let tx = get_f64(&op.operands[0]).unwrap_or(0.0);
                    let ty = get_f64(&op.operands[1]).unwrap_or(0.0);
                    translate_matrix(&mut state.text.line_matrix, tx, ty);
                    state.text.text_matrix = state.text.line_matrix;
                    if op.operator == "TD" {
                        state.text.leading = -ty;
                    }
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "T*" => {
                text_newline(&mut state.text);
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Tf" => {
                if op.operands.len() >= 2 {
                    state.text.font_name = op.operands[0].as_name().ok().map(|v| v.to_vec());
                    state.text.font_size = get_f64(&op.operands[1]).unwrap_or(0.0);
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Tc" => {
                if let Some(v) = op.operands.get(0).and_then(get_f64) {
                    state.text.char_space = v;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Tw" => {
                if let Some(v) = op.operands.get(0).and_then(get_f64) {
                    state.text.word_space = v;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Tz" => {
                if let Some(v) = op.operands.get(0).and_then(get_f64) {
                    state.text.h_scale = v / 100.0;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "TL" => {
                if let Some(v) = op.operands.get(0).and_then(get_f64) {
                    state.text.leading = v;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Ts" => {
                if let Some(v) = op.operands.get(0).and_then(get_f64) {
                    state.text.text_rise = v;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "'" => {
                text_newline(&mut state.text);
                let font = state
                    .text
                    .font_name
                    .as_ref()
                    .and_then(|name| get_font_metrics(doc, &resources.dict, name, &mut font_cache));
                if handle_text_show(&mut op, &mut state, font.as_ref(), masks) {
                    modified = true;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "\"" => {
                if op.operands.len() >= 3 {
                    if let Some(v) = get_f64(&op.operands[0]) {
                        state.text.word_space = v;
                    }
                    if let Some(v) = get_f64(&op.operands[1]) {
                        state.text.char_space = v;
                    }
                    text_newline(&mut state.text);
                }
                let font = state
                    .text
                    .font_name
                    .as_ref()
                    .and_then(|name| get_font_metrics(doc, &resources.dict, name, &mut font_cache));
                if handle_text_show(&mut op, &mut state, font.as_ref(), masks) {
                    modified = true;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "Tj" | "TJ" => {
                let font = state
                    .text
                    .font_name
                    .as_ref()
                    .and_then(|name| get_font_metrics(doc, &resources.dict, name, &mut font_cache));
                if handle_text_show(&mut op, &mut state, font.as_ref(), masks) {
                    modified = true;
                }
                push_op(op, &mut new_ops, &mut keep_flags);
            }
            "m" | "l" | "c" | "v" | "y" | "re" | "h" => {
                match op.operator.as_str() {
                    "m" | "l" => {
                        if op.operands.len() >= 2 {
                            let x = get_f64(&op.operands[0]).unwrap_or(0.0);
                            let y = get_f64(&op.operands[1]).unwrap_or(0.0);
                            let (tx, ty) = apply_matrix(state.ctm, x, y);
                            update_path_bbox(&mut path_bbox, tx, ty);
                        }
                    }
                    "c" => {
                        if op.operands.len() >= 6 {
                            for i in (0..6).step_by(2) {
                                let x = get_f64(&op.operands[i]).unwrap_or(0.0);
                                let y = get_f64(&op.operands[i + 1]).unwrap_or(0.0);
                                let (tx, ty) = apply_matrix(state.ctm, x, y);
                                update_path_bbox(&mut path_bbox, tx, ty);
                            }
                        }
                    }
                    "v" | "y" => {
                        if op.operands.len() >= 4 {
                            for i in (0..4).step_by(2) {
                                let x = get_f64(&op.operands[i]).unwrap_or(0.0);
                                let y = get_f64(&op.operands[i + 1]).unwrap_or(0.0);
                                let (tx, ty) = apply_matrix(state.ctm, x, y);
                                update_path_bbox(&mut path_bbox, tx, ty);
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
                                let (tx, ty) = apply_matrix(state.ctm, cx, cy);
                                update_path_bbox(&mut path_bbox, tx, ty);
                            }
                        }
                    }
                    "h" => {}
                    _ => {}
                }
                let idx = new_ops.len();
                push_op(op, &mut new_ops, &mut keep_flags);
                path_indices.push(idx);
            }
            "W" | "W*" => {
                push_op(op, &mut new_ops, &mut keep_flags);
                path_indices.clear();
                path_bbox = None;
            }
            "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" => {
                let intersects = path_bbox
                    .map(|(min_x, min_y, max_x, max_y)| {
                        masks.intersects_rect(min_x, min_y, max_x, max_y)
                    })
                    .unwrap_or(false);
                if intersects {
                    // マスクと交差するパスの構築操作を全て削除
                    for idx in path_indices.drain(..) {
                        keep_flags[idx] = false;
                    }
                    // ペイント操作自体も出力しない（pushしない）
                    path_bbox = None;
                    modified = true;
                } else {
                    push_op(op, &mut new_ops, &mut keep_flags);
                    path_indices.clear();
                    path_bbox = None;
                }
            }
            "n" => {
                push_op(op, &mut new_ops, &mut keep_flags);
                path_indices.clear();
                path_bbox = None;
            }
            "Do" => {
                let name = op
                    .operands
                    .get(0)
                    .and_then(|o| o.as_name().ok())
                    .map(|v| v.to_vec());
                let Some(name) = name else {
                    push_op(op, &mut new_ops, &mut keep_flags);
                    continue;
                };

                let xobj_dict = match resources.dict.get(b"XObject").and_then(|o| o.as_dict()) {
                    Ok(d) => d,
                    Err(_) => {
                        push_op(op, &mut new_ops, &mut keep_flags);
                        continue;
                    }
                };
                let xobj_entry = match xobj_dict.get(&name) {
                    Ok(obj) => obj,
                    Err(_) => {
                        push_op(op, &mut new_ops, &mut keep_flags);
                        continue;
                    }
                };

                let (xobj_stream, xobj_id) = match xobj_entry {
                    Object::Reference(id) => match doc.get_object(*id) {
                        Ok(obj) => match obj.as_stream() {
                            Ok(s) => (s.clone(), Some(*id)),
                            Err(_) => {
                                push_op(op, &mut new_ops, &mut keep_flags);
                                continue;
                            }
                        },
                        Err(_) => {
                            push_op(op, &mut new_ops, &mut keep_flags);
                            continue;
                        }
                    },
                    Object::Stream(s) => (s.clone(), None),
                    _ => {
                        push_op(op, &mut new_ops, &mut keep_flags);
                        continue;
                    }
                };

                let subtype = xobj_stream
                    .dict
                    .get(b"Subtype")
                    .and_then(|o| o.as_name_str())
                    .unwrap_or("");

                if subtype == "Image" {
                    // Compute image bbox in page space.
                    let pts = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
                    let mut min_x = f64::MAX;
                    let mut min_y = f64::MAX;
                    let mut max_x = f64::MIN;
                    let mut max_y = f64::MIN;
                    for (x, y) in pts {
                        let (tx, ty) = apply_matrix(state.ctm, x, y);
                        min_x = min_x.min(tx);
                        min_y = min_y.min(ty);
                        max_x = max_x.max(tx);
                        max_y = max_y.max(ty);
                    }
                    if !masks.intersects_rect(min_x, min_y, max_x, max_y) {
                        push_op(op, &mut new_ops, &mut keep_flags);
                        continue;
                    }

                    match redact_image_stream(&xobj_stream, state.ctm, masks) {
                        Ok(new_stream) => {
                            let new_id = doc.add_object(Object::Stream(new_stream));
                            let new_name = resources.add_xobject(new_id);
                            op.operands[0] = Object::Name(new_name);
                            push_op(op, &mut new_ops, &mut keep_flags);
                            modified = true;
                        }
                        Err(_) => {
                            // Fail-safe: drop the image instance if we can't safely redact it.
                            modified = true;
                        }
                    }
                } else if subtype == "Form" {
                    // Compute form matrix
                    let form_matrix = xobj_stream
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

                    let form_ctm = mat_mul(form_matrix, state.ctm);

                    let (bbox, bbox_known) = xobj_stream
                        .dict
                        .get(b"BBox")
                        .and_then(|o| o.as_array())
                        .ok()
                        .and_then(|arr| {
                            if arr.len() == 4 {
                                Some((
                                    [
                                        get_f64(&arr[0]).unwrap_or(0.0),
                                        get_f64(&arr[1]).unwrap_or(0.0),
                                        get_f64(&arr[2]).unwrap_or(0.0),
                                        get_f64(&arr[3]).unwrap_or(0.0),
                                    ],
                                    true,
                                ))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(([0.0, 0.0, 1.0, 1.0], false));

                    let corners = [
                        (bbox[0], bbox[1]),
                        (bbox[0], bbox[3]),
                        (bbox[2], bbox[1]),
                        (bbox[2], bbox[3]),
                    ];
                    let mut min_x = f64::MAX;
                    let mut min_y = f64::MAX;
                    let mut max_x = f64::MIN;
                    let mut max_y = f64::MIN;
                    for (x, y) in corners {
                        let (tx, ty) = apply_matrix(form_ctm, x, y);
                        min_x = min_x.min(tx);
                        min_y = min_y.min(ty);
                        max_x = max_x.max(tx);
                        max_y = max_y.max(ty);
                    }

                    if bbox_known && !masks.intersects_rect(min_x, min_y, max_x, max_y) {
                        push_op(op, &mut new_ops, &mut keep_flags);
                        continue;
                    }

                    if let Some(id) = xobj_id {
                        if recursion.contains(&id) {
                            // Avoid infinite recursion. Drop this instance.
                            modified = true;
                            continue;
                        }
                    }

                    let content_bytes = match decode_form_content(&xobj_stream) {
                        Ok(data) => data,
                        Err(_) => {
                            modified = true;
                            continue;
                        }
                    };

                    let mut content = Content::decode(&content_bytes)
                        .map_err(|_| anyhow::anyhow!("Failed to decode form content"))?;
                    // 間接参照を解決し、親リソースから継承
                    let form_resources = resolve_form_resources(
                        doc,
                        &xobj_stream.dict,
                        &resources.dict,
                    );

                    if let Some(id) = xobj_id {
                        recursion.push(id);
                    }

                    let result = process_ops(
                        content.operations,
                        GraphicsState {
                            ctm: form_ctm,
                            text: TextState::new(),
                        },
                        ResourceState::new(form_resources),
                        doc,
                        masks,
                        recursion,
                    )?;

                    if let Some(id) = xobj_id {
                        recursion.retain(|v| *v != id);
                    }

                    if result.modified {
                        content.operations = result.ops;
                        let encoded = Content::encode(&content)
                            .map_err(|_| anyhow::anyhow!("Failed to encode form content"))?;
                        let mut new_stream = xobj_stream.clone();
                        new_stream.set_plain_content(encoded);
                        new_stream
                            .compress()
                            .map_err(|_| anyhow::anyhow!("Failed to compress form"))?;
                        new_stream.dict.set("Resources", result.resources);

                        let new_id = doc.add_object(Object::Stream(new_stream));
                        let new_name = resources.add_xobject(new_id);
                        op.operands[0] = Object::Name(new_name);
                        push_op(op, &mut new_ops, &mut keep_flags);
                        modified = true;
                    } else {
                        push_op(op, &mut new_ops, &mut keep_flags);
                    }
                } else {
                    push_op(op, &mut new_ops, &mut keep_flags);
                }
            }
            _ => push_op(op, &mut new_ops, &mut keep_flags),
        }
    }

    // ストリーム末尾でペイント操作なしに残ったパス構築操作を安全に削除
    if !path_indices.is_empty() {
        for idx in path_indices.drain(..) {
            keep_flags[idx] = false;
        }
        modified = true;
    }

    let filtered_ops = new_ops
        .into_iter()
        .zip(keep_flags.into_iter())
        .filter_map(|(op, keep)| if keep { Some(op) } else { None })
        .collect::<Vec<_>>();

    Ok(ProcessResult {
        ops: filtered_ops,
        resources: resources.dict,
        modified,
    })
}

pub fn apply_masks(doc: &mut Document, masks: &[MaskRect]) -> Result<(), anyhow::Error> {
    let mut masks_by_page: HashMap<u32, Vec<MaskRect>> = HashMap::new();
    for mask in masks {
        masks_by_page.entry(mask.page).or_default().push(mask.clone());
    }

    let pages = doc.get_pages();

    for (page_num, page_masks) in masks_by_page {
        let Some(&page_id) = pages.get(&page_num) else { continue };

        let resources = build_inherited_page_resources(doc, page_id);

        let content_data = doc.get_and_decode_page_content(page_id);
        let mut content = match content_data {
            Ok(c) => c,
            Err(_) => Content { operations: vec![] },
        };

        // Wrap original content in q/Q to isolate graphics state.
        let mut wrapped_ops = Vec::with_capacity(content.operations.len() + 2);
        wrapped_ops.push(Operation::new("q", vec![]));
        wrapped_ops.extend(content.operations);
        wrapped_ops.push(Operation::new("Q", vec![]));

        let mask_set = MaskSet::new(&page_masks);
        let result = process_ops(
            wrapped_ops,
            GraphicsState::new(),
            ResourceState::new(resources),
            doc,
            &mask_set,
            &mut Vec::new(),
        )?;

        content.operations = result.ops;

        // Append overlay masks for visible redaction
        content.operations.push(Operation::new("q", vec![]));
        for mask in &page_masks {
            if mask.color == "white" {
                content
                    .operations
                    .push(Operation::new("rg", vec![1.0.into(), 1.0.into(), 1.0.into()]));
            } else {
                content
                    .operations
                    .push(Operation::new("rg", vec![0.0.into(), 0.0.into(), 0.0.into()]));
            }
            content.operations.push(Operation::new(
                "re",
                vec![mask.x.into(), mask.y.into(), mask.width.into(), mask.height.into()],
            ));
            content.operations.push(Operation::new("f", vec![]));
        }
        content.operations.push(Operation::new("Q", vec![]));

        let modified_content = content.encode()?;
        let mut new_stream = Stream::new(Dictionary::new(), modified_content);
        new_stream
            .compress()
            .map_err(|e| anyhow::anyhow!("Compression failed: {:?}", e))?;
        let new_content_id = doc.add_object(Object::Stream(new_stream));

        if let Ok(obj) = doc.get_object_mut(page_id) {
            if let Ok(page_dict) = obj.as_dict_mut() {
                page_dict.set("Contents", new_content_id);
                page_dict.set("Resources", result.resources);
            }
        }
    }

    doc.prune_objects();
    doc.compress();
    Ok(())
}
