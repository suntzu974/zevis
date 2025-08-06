mod models;
mod websocket;
mod app;

use app::NotificationApp;

fn main() {
    // Initialize console logging
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    
    log::info!("Starting WebSocket Notifications App");
    
    yew::Renderer::<NotificationApp>::new().render();
}
