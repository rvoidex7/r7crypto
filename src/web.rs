//! Web/WASM bootstrap.
//!
//! Trunk loads this cdylib from `index.html`. The page only needs a canvas with
//! id `r7crypto_canvas`; all UI code is shared with the native binary.

use crate::viz::R7CryptoApp;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

/// Entry point called by the generated wasm-bindgen glue.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no browser window"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("no browser document"))?;
    let canvas = document
        .get_element_by_id("r7crypto_canvas")
        .ok_or_else(|| JsValue::from_str("missing #r7crypto_canvas"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    spawn_local(async move {
        let options = eframe::WebOptions {
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        if let Err(error) = eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(|_cc| Ok(Box::new(R7CryptoApp::default()))),
            )
            .await
        {
            web_sys::console::error_1(&error);
        }
    });

    Ok(())
}
