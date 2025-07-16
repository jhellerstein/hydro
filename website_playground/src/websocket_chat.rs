use futures::channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures::stream::StreamExt;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, ErrorEvent, CloseEvent, Event};

// Chat message types - Simple format matching DFIR server
#[derive(Debug, Clone)]
pub enum ChatMessageType {
    SetName(String),
    ChatText(String),
    SystemMessage(String),
}

// WebSocket events for DFIR processing
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    Connected,
    Disconnected,
    Message(String),
    Error(String),
    SendMessage(String),
}

// Chat state management
#[derive(Debug, Clone, Default)]
pub struct ChatState {
    pub username: String,
    pub active_users: Vec<String>,
    pub is_connected: bool,
}

// JavaScript callback types
type JsCallback = js_sys::Function;

#[wasm_bindgen]
pub struct WebSocketChat {
    websocket: Option<WebSocket>,
    state: Rc<RefCell<ChatState>>,
    event_sender: Option<UnboundedSender<WebSocketEvent>>,
    
    // JavaScript callbacks
    on_connected: Option<JsCallback>,
    on_disconnected: Option<JsCallback>,
    on_error: Option<JsCallback>,
    on_message_received: Option<JsCallback>,
    on_message_sent: Option<JsCallback>,
    on_user_joined: Option<JsCallback>,
    on_user_left: Option<JsCallback>,
}

#[wasm_bindgen]
impl WebSocketChat {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        crate::utils::set_panic_hook();
        
        Self {
            websocket: None,
            state: Rc::new(RefCell::new(ChatState::default())),
            event_sender: None,
            on_connected: None,
            on_disconnected: None,
            on_error: None,
            on_message_received: None,
            on_message_sent: None,
            on_user_joined: None,
            on_user_left: None,
        }
    }
    
    #[wasm_bindgen]
    pub fn connect(&mut self, url: &str, username: &str) -> Result<(), JsValue> {
        web_sys::console::log_1(&format!("WASM: Attempting to connect to {}", url).into());
        
        if self.websocket.is_some() {
            return Err(JsValue::from_str("Already connected"));
        }
        
        let websocket = WebSocket::new(url)?;
        websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        
        // Update state
        {
            let mut state = self.state.borrow_mut();
            state.username = username.to_string();
            state.is_connected = false;
            state.active_users.clear();
        }
        
        // Create event channels for DFIR
        let (event_sender, event_receiver) = mpsc::unbounded();
        self.event_sender = Some(event_sender.clone());
        
        // Set up WebSocket event handlers
        self.setup_websocket_handlers(&websocket, event_sender.clone())?;
        
        // Store the websocket reference BEFORE creating the DFIR graph
        self.websocket = Some(websocket);
        
        // Create and start DFIR graph
        self.create_dfir_graph(event_receiver)?;
        
        web_sys::console::log_1(&"WASM: WebSocket created and handlers set up".into());
        
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn disconnect(&mut self) {
        if let Some(ws) = &self.websocket {
            let _ = ws.close();
        }
        self.websocket = None;
        self.event_sender = None;
        
        let mut state = self.state.borrow_mut();
        state.is_connected = false;
        state.active_users.clear();
    }
    
    #[wasm_bindgen]
    pub fn send_message(&self, content: &str) -> Result<(), JsValue> {
        if !self.state.borrow().is_connected {
            return Err(JsValue::from_str("Not connected"));
        }
        
        if let Some(sender) = &self.event_sender {
            sender.unbounded_send(WebSocketEvent::SendMessage(content.to_string()))
                .map_err(|e| JsValue::from_str(&format!("Failed to queue message: {}", e)))?;
        }
        
        Ok(())
    }
    
    #[wasm_bindgen]
    pub fn get_active_users(&self) -> Vec<String> {
        self.state.borrow().active_users.clone()
    }
    
    #[wasm_bindgen]
    pub fn is_connected(&self) -> bool {
        self.state.borrow().is_connected
    }
    
    // JavaScript callback setters
    #[wasm_bindgen]
    pub fn set_on_connected(&mut self, callback: &JsCallback) {
        self.on_connected = Some(callback.clone());
    }
    
    #[wasm_bindgen]
    pub fn set_on_disconnected(&mut self, callback: &JsCallback) {
        self.on_disconnected = Some(callback.clone());
    }
    
    #[wasm_bindgen]
    pub fn set_on_error(&mut self, callback: &JsCallback) {
        self.on_error = Some(callback.clone());
    }
    
    #[wasm_bindgen]
    pub fn set_on_message_received(&mut self, callback: &JsCallback) {
        self.on_message_received = Some(callback.clone());
    }
    
    #[wasm_bindgen]
    pub fn set_on_message_sent(&mut self, callback: &JsCallback) {
        self.on_message_sent = Some(callback.clone());
    }
    
    #[wasm_bindgen]
    pub fn set_on_user_joined(&mut self, callback: &JsCallback) {
        self.on_user_joined = Some(callback.clone());
    }
    
    #[wasm_bindgen]
    pub fn set_on_user_left(&mut self, callback: &JsCallback) {
        self.on_user_left = Some(callback.clone());
    }
}

impl WebSocketChat {
    fn setup_websocket_handlers(
        &self,
        websocket: &WebSocket,
        event_sender: UnboundedSender<WebSocketEvent>,
    ) -> Result<(), JsValue> {
        // OnOpen handler
        {
            let sender = event_sender.clone();
            let onopen_callback = Closure::wrap(Box::new(move |_event: Event| {
                web_sys::console::log_1(&"WASM: WebSocket onopen triggered".into());
                let _ = sender.unbounded_send(WebSocketEvent::Connected);
            }) as Box<dyn FnMut(Event)>);
            websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();
        }
        
        // OnMessage handler
        {
            let sender = event_sender.clone();
            let onmessage_callback = Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(txt) = event.data().dyn_into::<js_sys::JsString>() {
                    let message = String::from(txt);
                    let _ = sender.unbounded_send(WebSocketEvent::Message(message));
                }
            }) as Box<dyn FnMut(MessageEvent)>);
            websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();
        }
        
        // OnError handler
        {
            let sender = event_sender.clone();
            let onerror_callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
                web_sys::console::log_1(&format!("WASM: WebSocket error event: {:?}", event.message()).into());
                web_sys::console::log_1(&format!("WASM: Error type: {:?}", event.type_()).into());
                let _ = sender.unbounded_send(WebSocketEvent::Error("Connection error".to_string()));
            }) as Box<dyn FnMut(ErrorEvent)>);
            websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();
        }
        
        // OnClose handler
        {
            let sender = event_sender.clone();
            let onclose_callback = Closure::wrap(Box::new(move |event: CloseEvent| {
                web_sys::console::log_1(&format!("WASM: WebSocket close event - code: {}, reason: {}, clean: {}", 
                    event.code(), event.reason(), event.was_clean()).into());
                let _ = sender.unbounded_send(WebSocketEvent::Disconnected);
            }) as Box<dyn FnMut(CloseEvent)>);
            websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
            onclose_callback.forget();
        }
        
        Ok(())
    }
    
    fn create_dfir_graph(&mut self, mut event_receiver: UnboundedReceiver<WebSocketEvent>) -> Result<(), JsValue> {
        let state = self.state.clone();
        let websocket = self.websocket.as_ref().unwrap().clone();
        
        // Clone callbacks for use in the async task
        let on_connected = self.on_connected.clone();
        let on_disconnected = self.on_disconnected.clone();
        let on_error = self.on_error.clone();
        let on_message_received = self.on_message_received.clone();
        let on_message_sent = self.on_message_sent.clone();
        let on_user_joined = self.on_user_joined.clone();
        let on_user_left = self.on_user_left.clone();
        
        // Start processing WebSocket events
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(event) = event_receiver.next().await {
                match event {
                    WebSocketEvent::Connected => {
                        web_sys::console::log_1(&"WASM: Processing Connected event".into());
                        {
                            let mut state_ref = state.borrow_mut();
                            state_ref.is_connected = true;
                        }
                        
                        // Send name command to DFIR server
                        let username = {
                            let state_ref = state.borrow();
                            state_ref.username.clone()
                        };
                        let name_command = format!("/name {}", username);
                        if let Err(e) = websocket.send_with_str(&name_command) {
                            web_sys::console::log_1(&format!("Failed to send name command: {:?}", e).into());
                        }
                        
                        web_sys::console::log_1(&"WASM: Calling on_connected callback".into());
                        if let Some(callback) = &on_connected {
                            let _ = callback.call0(&JsValue::NULL);
                        }
                    },
                    WebSocketEvent::Disconnected => {
                        {
                            let mut state_ref = state.borrow_mut();
                            state_ref.is_connected = false;
                            state_ref.active_users.clear();
                        }
                        if let Some(callback) = &on_disconnected {
                            let _ = callback.call0(&JsValue::NULL);
                        }
                    },
                    WebSocketEvent::Message(msg) => {
                        // DFIR server sends plain text messages
                        // Format: "address: message" for chat messages
                        // or system messages like "Welcome, username!"
                        
                        if msg.contains(": ") {
                            // Chat message format: "127.0.0.1:12345: Hello world"
                            if let Some(colon_pos) = msg.find(": ") {
                                let sender_part = &msg[..colon_pos];
                                let content = &msg[colon_pos + 2..];
                                
                                // Extract just the address part as sender for now
                                let sender = if sender_part.contains(":") {
                                    sender_part.to_string()
                                } else {
                                    sender_part.to_string()
                                };
                                
                                if let Some(callback) = &on_message_received {
                                    let timestamp = js_sys::Date::now();
                                    let _ = callback.call3(
                                        &JsValue::NULL,
                                        &JsValue::from_str(content),
                                        &JsValue::from_str(&sender),
                                        &JsValue::from_f64(timestamp)
                                    );
                                }
                            }
                        } else if msg.starts_with("Welcome,") || msg.contains("joined") || msg.contains("left") {
                            // System message
                            if msg.contains("joined") {
                                // Extract username from "X joined the chat" 
                                if let Some(pos) = msg.find(" joined") {
                                    let username = &msg[..pos];
                                    let mut state_ref = state.borrow_mut();
                                    if !state_ref.active_users.contains(&username.to_string()) {
                                        state_ref.active_users.push(username.to_string());
                                    }
                                    drop(state_ref);
                                    
                                    if let Some(callback) = &on_user_joined {
                                        let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(username));
                                    }
                                }
                            } else if msg.contains("left") {
                                // Extract username from "X left the chat"
                                if let Some(pos) = msg.find(" left") {
                                    let username = &msg[..pos];
                                    {
                                        let mut state_ref = state.borrow_mut();
                                        state_ref.active_users.retain(|u| u != username);
                                    }
                                    
                                    if let Some(callback) = &on_user_left {
                                        let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(username));
                                    }
                                }
                            }
                            
                            // Show as system message
                            web_sys::console::log_1(&format!("System: {}", msg).into());
                        } else {
                            // Other system messages
                            web_sys::console::log_1(&format!("Server: {}", msg).into());
                        }
                    },
                    WebSocketEvent::Error(error) => {
                        if let Some(callback) = &on_error {
                            let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(&error));
                        }
                    },
                    WebSocketEvent::SendMessage(content) => {
                        let (is_connected, username) = {
                            let state_ref = state.borrow();
                            (state_ref.is_connected, state_ref.username.clone())
                        };
                        
                        if is_connected {
                            let timestamp = js_sys::Date::now();
                            
                            // Send plain text message to match DFIR server protocol
                            if let Err(e) = websocket.send_with_str(&content) {
                                web_sys::console::log_1(&format!("Send error: {:?}", e).into());
                                if let Some(callback) = &on_error {
                                    let _ = callback.call1(&JsValue::NULL, &JsValue::from_str("Failed to send message"));
                                }
                            } else {
                                if let Some(callback) = &on_message_sent {
                                    let _ = callback.call2(
                                        &JsValue::NULL,
                                        &JsValue::from_str(&content),
                                        &JsValue::from_f64(timestamp)
                                    );
                                }
                            }
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}

// Export the WebSocketChat class to JavaScript
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
