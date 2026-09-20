use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use winit::dpi::PhysicalSize;
use winit::window::Window;

/// There is no filesystem in the browser, so model bytes come over HTTP
pub(crate) async fn fetch_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
    let browser = web_sys::window().ok_or("no browser window")?;
    let response: web_sys::Response = JsFuture::from(browser.fetch_with_str(url)).await?.dyn_into()?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!("HTTP {}", response.status())));
    }

    let buffer = JsFuture::from(response.array_buffer()?).await?;

    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

pub(crate) fn sync_canvas_size(window: &Window) {
    let Some(browser) = web_sys::window() else { return };

    let (Ok(width), Ok(height)) = (browser.inner_width(), browser.inner_height()) else {
        return;
    };
    let (Some(width), Some(height)) = (width.as_f64(), height.as_f64()) else {
        return;
    };

    let scale = browser.device_pixel_ratio();
    let size = PhysicalSize::new((width * scale) as u32, (height * scale) as u32);

    if window.inner_size() != size {
        let _ = window.request_inner_size(size);
    }
}
