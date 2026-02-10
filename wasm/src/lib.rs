use wasm_bindgen::prelude::*;
use masking_tool_core::parser::load_pdf_from_bytes;
use masking_tool_core::masker::{apply_masks, MaskRect};

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[derive(serde::Deserialize)]
pub struct JsMaskRect {
    pub page: u32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: String,
}

#[wasm_bindgen]
pub fn sanitize_pdf(pdf_data: &[u8], masks_val: JsValue) -> Result<Vec<u8>, JsValue> {
    let masks: Vec<JsMaskRect> = serde_wasm_bindgen::from_value(masks_val)
        .map_err(|e| JsValue::from_str(&format!("Invalid mask data: {}", e)))?;

    // Convert to core MaskRect
    let core_masks: Vec<MaskRect> = masks.into_iter().map(|m| MaskRect {
        page: m.page,
        x: m.x,
        y: m.y,
        width: m.width,
        height: m.height,
        color: m.color,
    }).collect();

    let mut doc = load_pdf_from_bytes(pdf_data)
        .map_err(|e| JsValue::from_str(&format!("Failed to load PDF: {}", e)))?;

    apply_masks(&mut doc, &core_masks)
        .map_err(|e| JsValue::from_str(&format!("Failed to apply masks: {}", e)))?;

    // Save to bytes
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer)
        .map_err(|e| JsValue::from_str(&format!("Failed to save PDF: {}", e)))?;

    Ok(buffer)
}
