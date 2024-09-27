use std::{
    pin::Pin,
    sync::{
        Arc,
        Mutex,
    },
    task::{
        Context,
        Poll,
    },
};

use futures::Future;
use image::RgbaImage;
use tokio::sync::oneshot;
use wasm_bindgen::{
    closure::Closure,
    JsCast,
};
use web_sys::{
    ErrorEvent,
    Event,
    HtmlImageElement,
    OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};

#[derive(Debug, thiserror::Error)]
pub enum LoadImageError {
    #[error("{message}")]
    ErrorEvent { message: String },
}

/// Loads image data asynchronously using the browser image decoding.
///
/// Internally this creates an `<img>` tag and then grabs the loaded image's
/// data.
///
/// # TODO
///
/// - Change unwraps to expect or proper errors.
pub fn load_image(src: &str) -> LoadImage {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let image_element = HtmlImageElement::new().expect("failed to create HtmlImageElement");

    let onload_callback = {
        let image_element = image_element.clone();
        let tx = tx.clone();

        Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
            let size = [
                image_element.natural_width(),
                image_element.natural_height(),
            ];

            tracing::debug!(?size, "image loaded");

            let canvas = OffscreenCanvas::new(size[0], size[1]).unwrap();
            let context: OffscreenCanvasRenderingContext2d =
                canvas.get_context("2d").unwrap().unwrap().unchecked_into();
            context
                .draw_image_with_html_image_element(&image_element, 0.0, 0.0)
                .unwrap();
            let image_data = context
                .get_image_data(0.0, 0.0, size[0] as f64, size[1] as f64)
                .unwrap();

            // Data is stored as a one-dimensional array in the RGBA order, with integer
            // values between 0 and 255 (inclusive).
            let data = image_data.data().0;

            let image = RgbaImage::from_raw(size[0], size[1], data).unwrap();

            let mut tx = tx.lock().unwrap();
            if let Some(tx) = tx.take() {
                // an error indicates that the receiver has been dropped. we can ignore that.
                let _ = tx.send(Ok(image));
            }
        })
    };
    image_element.set_onload(Some(onload_callback.as_ref().unchecked_ref()));

    let onerror_callback = Closure::<dyn FnMut(ErrorEvent)>::new(move |event: ErrorEvent| {
        let message = event.message();
        tracing::warn!("failed to load image: {message}");

        let mut tx = tx.lock().unwrap();
        if let Some(tx) = tx.take() {
            // an error indicates that the receiver has been dropped. we can ignore that.
            let _ = tx.send(Err(LoadImageError::ErrorEvent { message }));
        }
    });

    image_element.set_src(src);

    LoadImage {
        image_element,
        onload_callback,
        onerror_callback,
        rx,
    }
}

/// Future returned by [`load_image`]. This resolves to the loaded image.
pub struct LoadImage {
    image_element: HtmlImageElement,
    onload_callback: Closure<dyn FnMut(Event)>,
    onerror_callback: Closure<dyn FnMut(ErrorEvent)>,
    rx: oneshot::Receiver<Result<RgbaImage, LoadImageError>>,
}

impl Drop for LoadImage {
    fn drop(&mut self) {
        self.image_element.set_onload(None);
        self.image_element.set_onerror(None);
    }
}

impl Future for LoadImage {
    type Output = Result<RgbaImage, LoadImageError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.rx).poll(cx).map(|result| {
            // the sender can only be dropped if the callbacks are dropped and those are in
            // this future.
            result.expect("sender dropped")
        })
    }
}
