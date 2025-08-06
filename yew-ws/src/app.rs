use yew::prelude::*;
use gloo::timers::callback::Interval;
use std::collections::VecDeque;

use crate::models::NotificationMessage;

#[function_component(NotificationApp)]
pub fn notification_app() -> Html {
    let ws_url = "ws://localhost:3000/ws";
    let messages = use_state(|| VecDeque::<NotificationMessage>::new());
    let connected = use_state(|| false);
    let auto_reconnect = use_state(|| true);
    let reconnect_interval = use_state(|| None::<Interval>);
    
    // Connection effect
    {
        let connected = connected.clone();
        let messages = messages.clone();
        let auto_reconnect = auto_reconnect.clone();
        let reconnect_interval = reconnect_interval.clone();
        
        use_effect_with((), move |_| {
            connect_websocket(ws_url, connected, messages, auto_reconnect, reconnect_interval);
            || ()
        });
    }
    
    // Toggle auto-reconnect
    let toggle_reconnect = {
        let auto_reconnect = auto_reconnect.clone();
        Callback::from(move |_| {
            auto_reconnect.set(!*auto_reconnect);
        })
    };
    
    // Clear messages
    let clear_messages = {
        let messages = messages.clone();
        Callback::from(move |_| {
            messages.set(VecDeque::new());
        })
    };
    
    // Format timestamp for display
    let format_time = |timestamp: &str| -> String {
        if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            parsed.format("%H:%M:%S").to_string()
        } else {
            timestamp.chars().take(8).collect()
        }
    };
    
    html! {
        <div class="notification-app">
            <header class="header">
                <h1>{"🔔 WebSocket Notifications - Yew"}</h1>
                <div class="controls">
                    <div class={format!("status {}", if *connected { "connected" } else { "disconnected" })}>
                        {if *connected { "🟢 Connected" } else { "🔴 Disconnected" }}
                    </div>
                    <label class="checkbox">
                        <input 
                            type="checkbox" 
                            checked={*auto_reconnect}
                            onchange={toggle_reconnect}
                        />
                        {"Auto-reconnect"}
                    </label>
                    <button onclick={clear_messages} class="clear-btn">
                        {"🗑️ Clear"}
                    </button>
                </div>
            </header>
            
            <main class="notifications">
                <div class="info-bar">
                    <span>{format!("Total messages: {}", messages.len())}</span>
                    <span>{format!("WebSocket URL: {}", ws_url)}</span>
                </div>
                
                <div class="messages-container">
                    {if messages.is_empty() {
                        html! {
                            <div class="empty-state">
                                <p>{"🎯 Waiting for notifications..."}</p>
                                <small>{"Create or delete users in the backend to see real-time notifications here."}</small>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="messages-list">
                                {for messages.iter().rev().enumerate().map(|(index, msg)| {
                                    match msg {
                                        NotificationMessage::UserNotification(notification) => {
                                            let event_color = match notification.event_type.as_str() {
                                                "user_created" => "success",
                                                "user_deleted" => "warning",
                                                _ => "info"
                                            };
                                            
                                            html! {
                                                <div key={index} class={format!("message notification {}", event_color)}>
                                                    <div class="message-header">
                                                        <span class="event-type">
                                                            {match notification.event_type.as_str() {
                                                                "user_created" => "👤➕ User Created",
                                                                "user_deleted" => "👤🗑️ User Deleted", 
                                                                _ => &notification.event_type
                                                            }}
                                                        </span>
                                                        <time class="timestamp">
                                                            {format_time(&notification.timestamp)}
                                                        </time>
                                                    </div>
                                                    <div class="message-content">
                                                        <div class="notification-message">
                                                            {&notification.message}
                                                        </div>
                                                        <div class="user-details">
                                                            <strong>{&notification.user_data.name}</strong>
                                                            <span class="email">{"("}{&notification.user_data.email}{")"}</span>
                                                            <span class="user-id">{"ID: "}{notification.user_data.id}</span>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }
                                        NotificationMessage::WsMessage(ws_msg) => {
                                            html! {
                                                <div key={index} class="message ws-message">
                                                    <div class="message-header">
                                                        <span class="event-type">{"💬 Message"}</span>
                                                        <time class="timestamp">
                                                            {format_time(&ws_msg.timestamp)}
                                                        </time>
                                                    </div>
                                                    <div class="message-content">
                                                        <div class="user-name">{&ws_msg.user}</div>
                                                        <div class="message-text">{&ws_msg.message}</div>
                                                    </div>
                                                </div>
                                            }
                                        }
                                        NotificationMessage::Connected => {
                                            html! {
                                                <div key={index} class="message system success">
                                                    <div class="message-content">
                                                        {"🟢 Connected to WebSocket server"}
                                                    </div>
                                                </div>
                                            }
                                        }
                                        NotificationMessage::Disconnected => {
                                            html! {
                                                <div key={index} class="message system warning">
                                                    <div class="message-content">
                                                        {"🔴 Disconnected from WebSocket server"}
                                                    </div>
                                                </div>
                                            }
                                        }
                                        NotificationMessage::Error(error) => {
                                            html! {
                                                <div key={index} class="message system error">
                                                    <div class="message-content">
                                                        {"❌ Error: "}{error}
                                                    </div>
                                                </div>
                                            }
                                        }
                                    }
                                })}
                            </div>
                        }
                    }}
                </div>
            </main>
        </div>
    }
}

fn connect_websocket(
    ws_url: &str,
    connected: UseStateHandle<bool>,
    messages: UseStateHandle<VecDeque<NotificationMessage>>,
    auto_reconnect: UseStateHandle<bool>,
    reconnect_interval: UseStateHandle<Option<Interval>>,
) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use web_sys::{WebSocket, MessageEvent, CloseEvent, ErrorEvent};
    
    log::info!("Connecting to WebSocket: {}", ws_url);
    
    match WebSocket::new(ws_url) {
        Ok(ws) => {
            // Clear any existing reconnect interval
            if reconnect_interval.is_some() {
                reconnect_interval.set(None);
            }
            
            // On open
            let connected_clone = connected.clone();
            let messages_clone = messages.clone();
            let on_open = Closure::wrap(Box::new(move |_| {
                log::info!("WebSocket connected");
                connected_clone.set(true);
                let mut msgs = (*messages_clone).clone();
                msgs.push_back(NotificationMessage::Connected);
                if msgs.len() > 100 {
                    msgs.pop_front();
                }
                messages_clone.set(msgs);
            }) as Box<dyn FnMut(JsValue)>);
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            on_open.forget();
            
            // On message
            let messages_clone = messages.clone();
            let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                    let text: String = text.into();
                    log::info!("Received message: {}", text);
                    
                    let mut msgs = (*messages_clone).clone();
                    
                    // Try to parse as UserNotification first
                    if let Ok(notification) = serde_json::from_str::<crate::models::UserNotification>(&text) {
                        msgs.push_back(NotificationMessage::UserNotification(notification));
                    } else if let Ok(ws_msg) = serde_json::from_str::<crate::models::WsMessage>(&text) {
                        msgs.push_back(NotificationMessage::WsMessage(ws_msg));
                    } else {
                        log::warn!("Could not parse message: {}", text);
                    }
                    
                    // Keep only last 100 messages
                    if msgs.len() > 100 {
                        msgs.pop_front();
                    }
                    messages_clone.set(msgs);
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            on_message.forget();
            
            // On close
            let connected_clone = connected.clone();
            let messages_clone = messages.clone();
            let auto_reconnect_clone = auto_reconnect.clone();
            let reconnect_interval_clone = reconnect_interval.clone();
            let ws_url_clone = ws_url.to_string();
            
            let on_close = Closure::wrap(Box::new(move |_: CloseEvent| {
                log::info!("WebSocket disconnected");
                connected_clone.set(false);
                let mut msgs = (*messages_clone).clone();
                msgs.push_back(NotificationMessage::Disconnected);
                if msgs.len() > 100 {
                    msgs.pop_front();
                }
                messages_clone.set(msgs);
                
                // Auto-reconnect if enabled
                if *auto_reconnect_clone {
                    log::info!("Attempting to reconnect in 3 seconds...");
                    let connected_clone2 = connected_clone.clone();
                    let messages_clone2 = messages_clone.clone();
                    let auto_reconnect_clone2 = auto_reconnect_clone.clone();
                    let reconnect_interval_clone2 = reconnect_interval_clone.clone();
                    let ws_url_clone2 = ws_url_clone.clone();
                    
                    let interval = Interval::new(3000, move || {
                        if *auto_reconnect_clone2 {
                            connect_websocket(
                                &ws_url_clone2, 
                                connected_clone2.clone(), 
                                messages_clone2.clone(),
                                auto_reconnect_clone2.clone(),
                                reconnect_interval_clone2.clone()
                            );
                        }
                    });
                    reconnect_interval_clone.set(Some(interval));
                }
            }) as Box<dyn FnMut(CloseEvent)>);
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
            on_close.forget();
            
            // On error
            let messages_clone = messages.clone();
            let connected_clone = connected.clone();
            let on_error = Closure::wrap(Box::new(move |_: ErrorEvent| {
                log::error!("WebSocket error");
                connected_clone.set(false);
                let mut msgs = (*messages_clone).clone();
                msgs.push_back(NotificationMessage::Error("Connection error".to_string()));
                if msgs.len() > 100 {
                    msgs.pop_front();
                }
                messages_clone.set(msgs);
            }) as Box<dyn FnMut(ErrorEvent)>);
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            on_error.forget();
        }
        Err(e) => {
            log::error!("Failed to create WebSocket: {:?}", e);
            let mut msgs = (*messages).clone();
            msgs.push_back(NotificationMessage::Error("Failed to create WebSocket".to_string()));
            messages.set(msgs);
        }
    }
}
