use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, console, UrlSearchParams};
use gloo_net::http::Request;

use crate::types::{AuthContext, GoogleAuthRequest, GoogleAuthResponse};

#[component]
pub fn GoogleCallback() -> impl IntoView {
    let navigate = use_navigate();
    let auth_context = expect_context::<RwSignal<AuthContext>>();
    let (status_message, set_status_message) = signal("Processing Google Calendar authentication...".to_string());
    let (is_processing, set_is_processing) = signal(true);

    // Clone navigate for use in different closures
    let navigate_for_effect = navigate.clone();
    let navigate_for_view = navigate.clone();

    // Handle the OAuth callback on component mount
    create_effect(move |_| {
        if is_processing.get() {
            let navigate_clone = navigate_for_effect.clone();
            spawn_local(async move {
                // Get the authorization code from URL parameters
                if let Some(window) = window() {
                    if let Ok(url) = window.location().href() {
                        console::log_1(&format!("DEBUG: Callback URL: {}", url).into());
                        
                        // Parse URL search parameters
                        if let Ok(url_obj) = web_sys::Url::new(&url) {
                            let search_params = url_obj.search_params();
                            
                            if let Some(auth_code) = search_params.get("code") {
                                console::log_1(&"DEBUG: Found authorization code".into());
                                
                                // Get auth token for API call
                                let auth = auth_context.get();
                                if let Some(token) = auth.token {
                                    // Send authorization code to backend
                                    match Request::post("http://localhost:8000/auth/google-calendar-callback")
                                        .header("Authorization", &format!("Bearer {}", token))
                                        .header("Content-Type", "application/json")
                                        .json(&GoogleAuthRequest { auth_code })
                                    {
                                        Ok(request) => {
                                            match request.send().await {
                                                Ok(response) => {
                                                    if response.ok() {
                                                        if let Ok(result) = response.json::<GoogleAuthResponse>().await {
                                                            if result.success {
                                                                set_status_message.set("✅ Google Calendar connected successfully! Redirecting to chat...".to_string());
                                                                console::log_1(&"DEBUG: Google Calendar authentication successful".into());
                                                                
                                                                // Redirect to chat after short delay
                                                                set_timeout(
                                                                    move || {
                                                                        let nav = navigate_clone.clone();
                                                                        nav("/chat", Default::default());
                                                                    },
                                                                    std::time::Duration::from_millis(2000),
                                                                );
                                                            } else {
                                                                set_status_message.set(format!("❌ Authentication failed: {}", result.message));
                                                            }
                                                        } else {
                                                            set_status_message.set("❌ Failed to parse response from server".to_string());
                                                        }
                                                    } else {
                                                        set_status_message.set(format!("❌ Server error: {}", response.status()));
                                                    }
                                                }
                                                Err(e) => {
                                                    console::log_1(&format!("DEBUG: Request failed: {:?}", e).into());
                                                    set_status_message.set("❌ Failed to connect to server".to_string());
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            console::log_1(&format!("DEBUG: Failed to create request: {:?}", e).into());
                                            set_status_message.set("❌ Failed to create request".to_string());
                                        }
                                    }
                                } else {
                                    set_status_message.set("❌ Not authenticated. Please login first.".to_string());
                                    // Redirect to login
                                    let nav_clone = navigate_clone.clone();
                                    set_timeout(
                                        move || {
                                            nav_clone("/login", Default::default());
                                        },
                                        std::time::Duration::from_millis(2000),
                                    );
                                }
                            } else if let Some(error) = search_params.get("error") {
                                console::log_1(&format!("DEBUG: OAuth error: {}", error).into());
                                set_status_message.set(format!("❌ Authentication cancelled or failed: {}", error));
                            } else {
                                console::log_1(&"DEBUG: No authorization code or error in URL".into());
                                set_status_message.set("❌ Invalid callback URL - missing authorization code".to_string());
                            }
                        }
                    }
                }
                
                set_is_processing.set(false);
            });
        }
    });

    view! {
        <div style="
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            background-color: #f8f9fa;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        ">
            <div style="
                background: white;
                padding: 40px;
                border-radius: 12px;
                box-shadow: 0 4px 20px rgba(0,0,0,0.1);
                text-align: center;
                max-width: 500px;
            ">
                <div style="
                    font-size: 3em;
                    margin-bottom: 20px;
                ">
                    {move || if is_processing.get() { "🔄" } else { "📅" }}
                </div>
                
                <h2 style="
                    color: #2c3e50;
                    margin-bottom: 20px;
                    font-size: 1.5em;
                ">"Google Calendar Integration"</h2>
                
                <p style="
                    color: #666;
                    line-height: 1.6;
                    margin-bottom: 30px;
                ">
                    {move || status_message.get()}
                </p>
                
                {move || {
                    let navigate_clone = navigate_for_view.clone();
                    if is_processing.get() {
                        view! {
                            <div style="
                                width: 40px;
                                height: 40px;
                                border: 4px solid #f3f3f3;
                                border-top: 4px solid #3498db;
                                border-radius: 50%;
                                animation: spin 1s linear infinite;
                                margin: 0 auto;
                            "></div>
                            <style>
                                "@keyframes spin {
                                    0% { transform: rotate(0deg); }
                                    100% { transform: rotate(360deg); }
                                }"
                            </style>
                        }.into_any()
                    } else {
                        view! {
                            <button 
                                style="
                                    background: #3498db;
                                    color: white;
                                    border: none;
                                    padding: 12px 24px;
                                    border-radius: 6px;
                                    cursor: pointer;
                                    font-size: 1em;
                                    font-weight: 500;
                                "
                                on:click=move |_| {
                                    let nav = navigate_clone.clone();
                                    nav("/chat", Default::default());
                                }
                            >
                                "Continue to Chat"
                            </button>
                        }.into_any()
                    }
                }}
            </div>
        </div>
    }
} 