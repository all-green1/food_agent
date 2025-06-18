use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
use gloo_net::http::{Request, Headers};
use wasm_bindgen_futures::spawn_local;
use web_sys::{console, window};

use crate::types::{AuthContext, ChatRequest, ChatResponse, ChatMessage, SessionMessagesResponse};

fn get_current_time() -> String {
    let date = js_sys::Date::new_0();
    let hours = date.get_hours();
    let minutes = date.get_minutes();

    format!{"{:02}:{:02}", hours, minutes}
}

#[component]
pub fn ChatPage() -> impl IntoView {
    let (input_message, set_input_message) = signal(String::new());
    let (messages, set_messages) = signal::<Vec<ChatMessage>>(Vec::new());
    let (is_loading, set_is_loading) = signal(false);
    let (session_id, set_session_id) = signal::<Option<String>>(None);
    let auth_context = expect_context::<RwSignal<AuthContext>>();
    let navigate = use_navigate();
    
    // Create a node reference for the input field
    let input_ref = NodeRef::<leptos::html::Input>::new();
    
    // Create a node reference for the messages container for auto-scroll
    let messages_container_ref = NodeRef::<leptos::html::Div>::new();

    // Debug: Check auth state when component loads
    console::log_1(&"DEBUG: ChatPage component loaded".into());
    let initial_auth = auth_context.get_untracked();
    console::log_1(&format!("DEBUG: Initial auth state - is_authenticated: {}", initial_auth.is_authenticated).into());

    // Check authentication on page load
    let navigate_for_auth = navigate.clone();
    create_effect(move |_| {
        let auth = auth_context.get();
        console::log_1(&format!("DEBUG: Auth effect - is_authenticated: {}", auth.is_authenticated).into());
        
        if !auth.is_authenticated {
            console::log_1(&"DEBUG: Not authenticated, redirecting to login".into());
            let nav = navigate_for_auth.clone();
            nav("/login", Default::default());
        }
    });

    // Generate session ID when component loads (only once)
    let stored_session = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item("session_id").ok().flatten());

    if let Some(stored) = stored_session {
        set_session_id.set(Some(stored));
    } else {
        let timestamp = js_sys::Date::now() as u64;
        let random = (js_sys::Math::random() * 1000000.0) as u64;
        let new_session_id = format!("chat_{}_{}", timestamp, random);
        set_session_id.set(Some(new_session_id.clone()));
        
        console::log_1(&format!("DEBUG: Generated new session ID: {}", new_session_id).into());

        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten()) {
            storage.set_item("session_id", &new_session_id).ok();
        }
    }

    // Load conversation history when session_id changes
    create_effect(move |_| {
        if let Some(current_session) = session_id.get() {
            let auth = auth_context.get_untracked();
            
            console::log_1(&format!("DEBUG: Loading conversation history for session: {}", current_session).into());
            
            if auth.is_authenticated && auth.token.is_some() {
                let token = auth.token.clone().unwrap();
                let session_for_fetch = current_session.clone();
                let set_messages_clone = set_messages.clone(); // Properly capture the signal
                
                console::log_1(&format!("DEBUG: Token being sent: {}", &token[..20.min(token.len())]).into()); // Show first 20 chars of token
                
                spawn_local(async move {
                    // Create headers with authorization
                    let mut headers = Headers::new();
                    headers.set("Authorization", &format!("Bearer {}", token));
                    headers.set("Content-Type", "application/json");

                    let request = Request::get(&format!("http://localhost:8000/get_session_messages/{}", session_for_fetch))
                        .headers(headers);

                    match request.send().await {
                        Ok(response) => {
                            if response.ok() {
                                console::log_1(&"DEBUG: Successfully fetched conversation history".into());
                                
                                // Parse the response using proper typed deserialization
                                if let Ok(session_messages) = response.json::<SessionMessagesResponse>().await {
                                    console::log_1(&format!("DEBUG: Received {} messages", session_messages.messages.len()).into());
                                    
                                    // Convert to ChatMessage format and update the messages signal
                                    let mut chat_messages = Vec::new();
                                    
                                    for msg in session_messages.messages {
                                        // Add user message
                                        chat_messages.push(ChatMessage {
                                            content: msg.message,
                                            is_user: true,
                                            timestamp: msg.created_at.clone(),
                                        });
                                        
                                        // Add assistant response
                                        chat_messages.push(ChatMessage {
                                            content: msg.response,
                                            is_user: false,
                                            timestamp: msg.created_at,
                                        });
                                    }
                                    
                                    console::log_1(&format!("DEBUG: Converted {} messages", chat_messages.len()).into());
                                    set_messages_clone.set(chat_messages); // Use the properly captured signal
                                }
                            } else {
                                console::log_1(&format!("DEBUG: Failed to fetch conversation history: {}", response.status()).into());
                                
                                // If unauthorized, clear auth and redirect to login
                                if response.status() == 401 {
                                    console::log_1(&"DEBUG: Token expired/invalid, clearing auth".into());
                                    AuthContext::clear_from_storage();
                                    // Note: We can't easily redirect from here as we'd need navigate in scope
                                }
                            }
                        }
                        Err(e) => {
                            console::log_1(&format!("DEBUG: Error fetching conversation history: {}", e).into());
                        }
                    }
                });
            } else {
                console::log_1(&"DEBUG: User not authenticated, skipping conversation history load".into());
            }
        }
    });

    // Handle logout
    let navigate_for_logout = navigate.clone();
    let handle_logout = move |_| {
        // Clear localStorage
        AuthContext::clear_from_storage();
        
        // Clear auth context
        auth_context.set(AuthContext::default());
        
        // Navigate to login
        let nav = navigate_for_logout.clone();
        nav("/login", Default::default());
    };

    // Handle new chat
    let handle_new_chat = move |_| {
        // Clear current session
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten()) 
        {
            let _ = storage.remove_item("session_id");
        }
        
        // Generate new session
        let new_session = format!("session_{}", js_sys::Math::random().to_string().replace("0.", ""));
        set_session_id.set(Some(new_session.clone()));
        
        // Save new session to localStorage
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten()) 
        {
            let _ = storage.set_item("session_id", &new_session);
        }
        
        // Clear messages
        set_messages.set(Vec::new());
    };

    // Auto-focus input field effect
    create_effect(move |_| {
        // Focus when loading finishes
        if !is_loading.get() {
            if let Some(input_element) = input_ref.get() {
                let _ = input_element.focus();
            }
        }
    });

    // Auto-scroll to bottom when messages change
    create_effect(move |_| {
        let _ = messages.get(); // Track messages changes
        let _ = is_loading.get(); // Also track loading state changes
        
        // Scroll to bottom - this will run after the reactive update
        if let Some(container) = messages_container_ref.get() {
            container.set_scroll_top(container.scroll_height());
        }
    });

    view! {
        <div style="
            display: flex;
            height: 100vh;
            width: 100vw;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: #f8f9fa;
            margin: 0;
            padding: 0;
            overflow: hidden;
        ">
            // Sidebar
            <div style="
                width: 280px;
                background-color: #2c3e50;
                color: white;
                padding: 20px;
                display: flex;
                flex-direction: column;
                box-shadow: 2px 0 10px rgba(0,0,0,0.1);
            ">
                <h2 style="
                    margin: 0 0 30px 0;
                    font-size: 1.5em;
                    font-weight: 600;
                    color: #ecf0f1;
                ">"Food Agent Chat"</h2>
                
                <button 
                    style="
                        background: linear-gradient(135deg, #3498db, #2980b9);
                        color: white;
                        border: none;
                        padding: 12px 20px;
                        border-radius: 8px;
                        cursor: pointer;
                        font-size: 1em;
                        font-weight: 500;
                        transition: all 0.3s ease;
                        box-shadow: 0 2px 4px rgba(0,0,0,0.2);
                        margin-bottom: 20px;
                    "
                    on:click=handle_new_chat>
                    " New Conversation"
                </button>
                
                <div style="
                    background-color: rgba(255,255,255,0.1);
                    padding: 15px;
                    border-radius: 8px;
                    margin-top: auto;
                ">
                    <div style="font-size: 0.85em; opacity: 0.8; margin-bottom: 8px;">"Current Session:"</div>
                    <div style="
                        font-size: 0.75em;
                        font-family: monospace;
                        word-break: break-all;
                        opacity: 0.6;
                    ">
                        {move || session_id.get().unwrap_or_else(|| "No session".to_string())}
                    </div>
                </div>
            </div>
            
            // Main chat area
            <div style="
                flex: 1;
                display: flex;
                flex-direction: column;
                height: 100vh;
                background-color: white;
            ">
                // Header
                <header style="
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 20px 30px;
                    background: white;
                    border-bottom: 1px solid #e1e5e9;
                    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                ">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <div style="font-size: 2em;">"🍎"</div>
                        <div>
                            <h1 style="margin: 0; font-size: 1.5em; color: #2c3e50;">"Food Agent"</h1>
                            <p style="margin: 0; font-size: 0.9em; color: #7f8c8d;">"AI-Powered Food Management"</p>
                        </div>
                    </div>
                    
                    <div style="display: flex; align-items: center; gap: 15px;">
                        {move || {
                            let auth = auth_context.get();
                            if let Some(username) = auth.username {
                                Some(view! {
                                    <span style="color: #555; font-size: 0.9em;">
                                        "Welcome, " {username}
                                    </span>
                                })
                            } else {
                                None
                            }
                        }}
                        
                        <button
                            on:click=handle_logout
                            style="
                                background: linear-gradient(135deg, #e74c3c, #c0392b);
                                color: white;
                                border: none;
                                padding: 8px 16px;
                                border-radius: 8px;
                                cursor: pointer;
                                font-size: 0.9em;
                                font-weight: 500;
                                transition: all 0.2s ease;
                            "
                            onmouseover="this.style.transform='translateY(-1px)'; this.style.boxShadow='0 4px 12px rgba(231, 76, 60, 0.3)'"
                            onmouseout="this.style.transform='translateY(0)'; this.style.boxShadow='none'"
                        >
                            "Logout"
                        </button>
                    </div>
                </header>
                
                // Chat messages area
                <div 
                    node_ref=messages_container_ref
                    style="
                        flex: 1;
                        overflow-y: auto;
                        padding: 20px;
                        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                        background-attachment: fixed;
                        scroll-behavior: smooth;
                    ">
                    <ChatDisplay messages=messages is_loading=is_loading />
                </div>
                
                // Input area at bottom
                <div style="
                    background-color: white;
                    border-top: 1px solid #e9ecef;
                    padding: 20px;
                    box-shadow: 0 -2px 10px rgba(0,0,0,0.1);
                ">
                    {
                        let navigate_for_form = navigate.clone();
                        view! {
                            <form on:submit=move |ev| {
                                ev.prevent_default();
                                let message = input_message.get();
                                if !message.trim().is_empty() {
                                    let auth = auth_context.get();
                                    
                                    console::log_1(&format!("DEBUG: Auth check - is_authenticated: {}", auth.is_authenticated).into());
                                    console::log_1(&format!("DEBUG: Auth check - username: {:?}", auth.username).into());
                                    console::log_1(&format!("DEBUG: Auth check - token present: {}", auth.token.is_some()).into());
                                    
                                    if !auth.is_authenticated {
                                        console::log_1(&"DEBUG: Not authenticated, redirecting to login".into());
                        
                                        let nav = navigate_for_form.clone();
                                        nav("/login", Default::default());
                                        return;
                                    }

                            let user_message = ChatMessage {
                                content: message.clone(),
                                is_user: true,
                                timestamp: get_current_time(),
                            };
                            set_is_loading.set(true);
                            set_messages.update(|msgs| msgs.push(user_message));

                            set_input_message.set(String::new());
                            
                            // Focus the input field after clearing it
                            if let Some(input_element) = input_ref.get() {
                                let _ = input_element.focus();
                            }

                            // Get input element reference before async block for later use
                            let input_element_for_focus = input_ref.get(); // Get the actual element, not the signal

                            spawn_local(async move {
                                let chat_request = ChatRequest {
                                    message: message.clone(),
                                    session_id: session_id.get_untracked(),
                                };

                                // Create headers and add Authorization if available
                                let mut headers = Headers::new();
                                if let Some(token) = auth.token.clone() {
                                    console::log_1(&format!("DEBUG: Chat request token: {}", &token[..20.min(token.len())]).into());
                                    headers.set("Authorization", &format!("Bearer {}", token));
                                }
                                headers.set("Content-Type", "application/json");

                                let request = Request::post("http://localhost:8000/chat")
                                    .headers(headers)
                                    .json(&chat_request)
                                    .unwrap();

                                match request.send().await {
                                    Ok(response) => {
                                        if response.ok() {  // Check if status is 200-299
                                            if let Ok(chat_response) = response.json::<ChatResponse>().await {
                                                let assistant_message = ChatMessage {
                                                    content: chat_response.response,
                                                    is_user: false,
                                                    timestamp: get_current_time(),
                                                };
                                                set_messages.update(|msgs| msgs.push(assistant_message));
                                            } else {
                                                let error_message = ChatMessage {
                                                    content: format!("Error: {}", response.status_text()),
                                                    is_user: false,
                                                    timestamp: get_current_time(),
                                                };
                                                set_messages.update(|msgs| msgs.push(error_message));
                                            }
                                        } else {
                                            // Handle error responses
                                            let error_message = ChatMessage {
                                                content: format!("Error: {}", response.status_text()),
                                                is_user: false,
                                                timestamp: get_current_time(),
                                            };
                                            set_messages.update(|msgs| msgs.push(error_message));
                                            
                                            // If unauthorized, clear auth
                                            if response.status() == 401 {
                                                console::log_1(&"DEBUG: Token expired/invalid during chat, clearing auth".into());
                                                AuthContext::clear_from_storage();
                                            }
                                        }
                                    }
                                    Err(e) => {
                                       let error_message = ChatMessage {
                                        content: format!("Network error: {}", e),
                                        is_user: false,
                                        timestamp: get_current_time(),
                                       };
                                       set_messages.update(|msgs| msgs.push(error_message)); 
                                    }
                                }
                                set_is_loading.set(false);

                                // Re-focus the input field after receiving response
                                if let Some(input_element) = input_element_for_focus {
                                    let _ = input_element.focus();
                                }
                            });
                        }
                    }>
                        <div style="
                            display: flex;
                            gap: 12px;
                            align-items: flex-end;
                            max-width: 1000px;
                            margin: 0 auto;
                        ">
                            <input 
                                node_ref=input_ref
                                type="text"
                                placeholder="Type your message here..."
                                autofocus=true
                                on:input:target=move |ev| {
                                    set_input_message.set(ev.target().value());
                                }
                                prop:value=move || input_message.get()
                                style="
                                    flex: 1;
                                    padding: 16px 20px;
                                    border: 2px solid #e9ecef;
                                    border-radius: 25px;
                                    font-size: 1em;
                                    outline: none;
                                    transition: all 0.3s ease;
                                    background-color: #f8f9fa;
                                "
                                disabled=move || is_loading.get()
                            />
                            <button 
                                type="submit"
                                disabled=move || is_loading.get() || input_message.get().trim().is_empty()
                                style="
                                    background: linear-gradient(135deg, #667eea, #764ba2);
                                    color: white;
                                    border: none;
                                    padding: 16px 24px;
                                    border-radius: 25px;
                                    cursor: pointer;
                                    font-size: 1em;
                                    font-weight: 500;
                                    transition: all 0.3s ease;
                                    box-shadow: 0 2px 10px rgba(102, 126, 234, 0.3);
                                    min-width: 100px;
                                "
                            >
                                {move || if is_loading.get() { "Sending..." } else { "Send 📤" }}
                            </button>
                            </div>
                        </form>
                        }
                    }
                </div>
            </div>
        </div>
    }
}

#[component]
fn ChatDisplay(messages: ReadSignal<Vec<ChatMessage>>,
    is_loading: ReadSignal<bool>
) -> impl IntoView {
    view! {
        <div style="
            max-width: 1000px;
            margin: 0 auto;
            padding: 20px 0;
        ">
            {move || {
                if messages.get().is_empty() && !is_loading.get() {
                    Some(view! {
                        <div style="
                            text-align: center;
                            color: rgba(255,255,255,0.8);
                            font-size: 1.2em;
                            margin-top: 100px;
                        ">
                            <div style="font-size: 3em; margin-bottom: 20px;">"🍎"</div>
                            <div>"Welcome to Food Agent Chat!"</div>
                            <div style="font-size: 0.9em; margin-top: 10px; opacity: 0.7;">
                                "Start by telling me about the food you'd like to add to your inventory."
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}
            
            <For
                each=move || messages.get()
                key=|message| format!("{}_{}", message.timestamp, message.content.len())
                children=move |message: ChatMessage| {
                    let (message_style, container_style) = if message.is_user {
                        (
                            "
                                background: linear-gradient(135deg, #667eea, #764ba2);
                                color: white;
                                margin-left: auto;
                                margin-right: 0;
                            ",
                            "justify-content: flex-end;"
                        )
                    } else {
                        (
                            "
                                background-color: rgba(255,255,255,0.95);
                                color: #333;
                                margin-left: 0;
                                margin-right: auto;
                                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                            ",
                            "justify-content: flex-start;"
                        )
                    };
                    
                    view! {
                        <div style={format!("
                            display: flex;
                            margin: 16px 0;
                            {}
                        ", container_style)}>
                            <div style={format!("
                                max-width: 70%;
                                padding: 16px 20px;
                                border-radius: 20px;
                                word-wrap: break-word;
                                {}
                            ", message_style)}>
                                <div style="font-size: 1em; line-height: 1.4;">
                                    {message.content}
                                </div>
                                <div style="
                                    font-size: 0.75em;
                                    opacity: 0.7;
                                    margin-top: 8px;
                                    text-align: right;
                                ">
                                    {if message.is_user { "You" } else { "🤖 Assistant" }} 
                                    " • " {message.timestamp}
                                </div>
                            </div>
                        </div>
                    }
                }
            />
            
            // Show loading indicator
            {move || {
                if is_loading.get() {
                   Some(view! {
                        <div style="display: flex; justify-content: flex-start; margin: 16px 0;">
                            <div style="
                                background-color: rgba(255,255,255,0.95);
                                color: #666;
                                max-width: 70%;
                                padding: 16px 20px;
                                border-radius: 20px;
                                font-style: italic;
                                box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                                animation: pulse 1.5s ease-in-out infinite alternate;
                            ">
                                "🤖 Assistant is thinking..."
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}