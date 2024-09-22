#![allow(dead_code)]

pub mod app;
pub mod error;
pub mod scene;
pub mod utils;

use wasm_bindgen::JsCast;

use crate::app::App;

fn main() {
    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();

    tracing::info!("starting app");

    let root = web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
        .get_element_by_id("root")
        .expect("no root element")
        .dyn_into()
        .unwrap();

    leptos::mount_to(root, App);
}
