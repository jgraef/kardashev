use std::{
    fmt::{
        Debug,
        Display,
    },
    ops::Deref,
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll,
    },
};

use futures::Future;
use gloo_file::{
    Blob,
    ObjectUrl,
};
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

use crate::utils::thread_local_cell::ThreadLocalCell;

#[derive(Debug, thiserror::Error)]
#[error("load image error: {url}")]
pub struct LoadImageError {
    pub url: ImageUrl,
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

#[derive(Clone)]
pub enum ImageUrl {
    Url(Url),
    ObjectUrl(Arc<ThreadLocalCell<ObjectUrl>>),
}

impl ImageUrl {
    fn as_str(&self) -> &str {
        match self {
            Self::Url(url) => url.as_str(),
            Self::ObjectUrl(url) => url.get().as_ref(),
        }
    }
}

impl From<Url> for ImageUrl {
    fn from(value: Url) -> Self {
        Self::Url(value)
    }
}

impl From<ObjectUrl> for ImageUrl {
    fn from(value: ObjectUrl) -> Self {
        Self::ObjectUrl(Arc::new(ThreadLocalCell::new(value)))
    }
}

impl From<Blob> for ImageUrl {
    fn from(value: Blob) -> Self {
        ObjectUrl::from(value).into()
    }
}

impl From<web_sys::Blob> for ImageUrl {
    fn from(value: web_sys::Blob) -> Self {
        Blob::from(value).into()
    }
}

impl Display for ImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(url) => write!(f, "{url}"),
            Self::ObjectUrl(url) => write!(f, "{}", url.get().deref()),
        }
    }
}

impl Debug for ImageUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct Helper<'a>(&'a str);
        impl<'a> Debug for Helper<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        match self {
            Self::Url(url) => f.debug_tuple("Url").field(&Helper(url.as_str())).finish(),
            Self::ObjectUrl(url) => {
                f.debug_tuple("ObjectUrl")
                    .field(&Helper(url.get().deref()))
                    .finish()
            }
        }
    }
}

pub fn load_image(url: impl Into<ImageUrl>) -> LoadImage {
    let url = url.into();
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
        _onload_callback: onload_callback,
        _onerror_callback: onerror_callback,
        rx,
    }
}

/// Future returned by [`load_image`]. This resolves to the loaded image.
pub struct LoadImage {
    url: ImageUrl,
    image_element: HtmlImageElement,
    _onload_callback: Closure<dyn FnMut(Event)>,
    _onerror_callback: Closure<dyn FnMut(Event)>,
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
