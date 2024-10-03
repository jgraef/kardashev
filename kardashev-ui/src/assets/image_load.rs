use std::{
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll,
    },
};

use futures::Future;
use image::RgbaImage;
use kardashev_client::AssetClient;
use parking_lot::Mutex;
use tokio::sync::oneshot;
use url::Url;
use wasm_bindgen::{
    closure::Closure,
    JsCast,
};
use web_sys::{
    Event,
    HtmlImageElement,
    OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};

#[derive(Debug, thiserror::Error)]
#[error("load image error")]
pub struct LoadImageError {
    pub url: Url,
    pub reason: LoadImageErrorReason,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadImageErrorReason {
    #[error("error event")]
    ErrorEvent,
    #[error("failed to decode image")]
    DecodeError,
}

pub trait AssetClientLoadImageExt {
    fn load_image(&self, url: &str) -> LoadImage;
}

impl AssetClientLoadImageExt for AssetClient {
    /// Loads image data using the browser image decoding.
    fn load_image(&self, url: &str) -> LoadImage {
        tracing::debug!(asset_url = %self.asset_url(), %url, "building image url");
        let url = self.asset_url().join(url).unwrap();
        load_image(url)
    }
}

fn load_image(url: Url) -> LoadImage {
    tracing::debug!(%url, "loading image");

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

            let result = RgbaImage::from_raw(size[0], size[1], data)
                .ok_or(LoadImageErrorReason::DecodeError);

            let mut tx = tx.lock();
            if let Some(tx) = tx.take() {
                // an error indicates that the receiver has been dropped. we can ignore that.
                let _ = tx.send(result);
            }
        })
    };
    image_element.set_onload(Some(onload_callback.as_ref().unchecked_ref()));

    let onerror_callback = Closure::<dyn FnMut(Event)>::new(move |_event: Event| {
        //let message = event.message();
        let mut tx = tx.lock();
        if let Some(tx) = tx.take() {
            // an error indicates that the receiver has been dropped. we can ignore that.
            let _ = tx.send(Err(LoadImageErrorReason::ErrorEvent));
        }
    });
    image_element.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

    image_element.set_src(url.as_str());

    LoadImage {
        url,
        image_element,
        onload_callback,
        onerror_callback,
        rx,
    }
}

/// Future returned by [`load_image`]. This resolves to the loaded image.
pub struct LoadImage {
    url: Url,
    image_element: HtmlImageElement,
    onload_callback: Closure<dyn FnMut(Event)>,
    onerror_callback: Closure<dyn FnMut(Event)>,
    rx: oneshot::Receiver<Result<RgbaImage, LoadImageErrorReason>>,
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
            result.expect("sender dropped").map_err(|reason| {
                let error = LoadImageError {
                    url: self.url.clone(),
                    reason,
                };
                tracing::error!(url = %self.url, ?error, "image load failed");
                error
            })
        })
    }
}
