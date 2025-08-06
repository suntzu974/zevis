mod models;
mod app;

use app::NotificationApp;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    // Initialize console logging
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    
    log::info!("Starting WebSocket Notifications App");
    
    yew::Renderer::<NotificationApp>::new().render();
}
