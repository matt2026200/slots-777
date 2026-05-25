use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub fn get_canvas_context(canvas_id: &str) -> Option<CanvasRenderingContext2d> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let canvas = document.get_element_by_id(canvas_id)?;
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>().ok()?;

    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}
