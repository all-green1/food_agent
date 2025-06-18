use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

use crate::types::{AuthContext, LoginRequest, LoginResponse};

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (pwd, set_pwd) = signal(String::new());
    let (is_logged_in, set_is_logged_in) = signal::<Option<LoginResponse>>(None);
    let (is_sent, set_is_sent) = signal(false);
    let (error_message, set_error_message) = signal(String::new());
    
    let auth_context = expect_context::<RwSignal<AuthContext>>();
    let navigate = use_navigate();
    
    view! {
        <div style="
            display: flex;
            height: 100vh;
            width: 100vw;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            margin: 0;
            padding: 0;
            overflow: hidden;
            align-items: center;
            justify-content: center;
        ">
            // Login Card
            <div style="
                background-color: rgba(255, 255, 255, 0.95);
                backdrop-filter: blur(10px);
                border-radius: 20px;
                padding: 40px;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
                width: 100%;
                max-width: 400px;
                margin: 20px;
            ">
                // Header
                <div style="text-align: center; margin-bottom: 40px;">
                    <div style="
                        font-size: 3em;
                        margin-bottom: 15px;
                    ">"🍎"</div>
                    <h1 style="
                        margin: 0 0 10px 0;
                        font-size: 2em;
                        font-weight: 600;
                        background: linear-gradient(135deg, #667eea, #764ba2);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        background-clip: text;
                    ">"Food Agent"</h1>
                    <p style="
                        margin: 0;
                        color: #666;
                        font-size: 1.1em;
                    ">"Sign in to your account"</p>
                </div>

                // Success Message
                {move || {
                    if let Some(username) = auth_context.get().username {
                        if !username.is_empty() {
                            Some(view! {
                                <div style="
                                    background: linear-gradient(135deg, #4CAF50, #45a049);
                                    color: white;
                                    padding: 15px 20px;
                                    border-radius: 12px;
                                    margin-bottom: 20px;
                                    text-align: center;
                                    box-shadow: 0 4px 12px rgba(76, 175, 80, 0.3);
                                ">
                                    <div style="font-weight: 500;">
                                        "Welcome back, "{username}"! 🎉"
                                    </div>
                                    <div style="font-size: 0.9em; opacity: 0.9; margin-top: 5px;">
                                        "Login successful"
                                    </div>
                                </div>
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }}

                // Error Message
                {move || {
                    if !error_message.get().is_empty() {
                        Some(view! {
                            <div style="
                                background: linear-gradient(135deg, #f44336, #d32f2f);
                                color: white;
                                padding: 15px 20px;
                                border-radius: 12px;
                                margin-bottom: 20px;
                                text-align: center;
                                box-shadow: 0 4px 12px rgba(244, 67, 54, 0.3);
                            ">
                                <div style="font-weight: 500;">
                                    "⚠️ " {error_message.get()}
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }
                }}
                
                // Login Form
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    let password = pwd.get();
                    let user = username.get();
                    if !&user.is_empty() || !&password.is_empty() {
                        set_is_sent.set(true);
                        set_error_message.set(String::new()); // Clear previous errors
                        
                        // Clone navigate before async move
                        let nav = navigate.clone();
                        
                        spawn_local(async move {
                            let login_details = LoginRequest {
                                email: user.clone(), 
                                password: password.clone(),
                            };
                            match Request::post("http://localhost:8000/auth/login")
                                .json(&login_details)
                                .unwrap()
                                .send()
                                .await
                            {
                                Ok(response) => {
                                    if response.ok() {  // Check if status is 200-299
                                        match response.json::<LoginResponse>().await {
                                            Ok(login_response) => {
                                                set_is_logged_in.set(Some(login_response.clone()));
                                                let new_auth_context = AuthContext {
                                                    token: Some(login_response.access_token.clone()),
                                                    user_id: Some(login_response.user_id),
                                                    username: Some(login_response.username.clone()),
                                                    is_authenticated: true,
                                                };
                                                
                                                // Save to localStorage before setting the context
                                                new_auth_context.save_to_storage();
                                                auth_context.set(new_auth_context);
                                                
                                                console::log_1(&"DEBUG: Login successful, auth context set and saved to localStorage".into());
                                                console::log_1(&format!("DEBUG: Set username: {}", login_response.username).into());
                                                console::log_1(&"DEBUG: Set is_authenticated: true".into());
                                                console::log_1(&"DEBUG: About to navigate to /chat".into());
                                                
                                                nav("/chat", Default::default());
                                            }
                                            Err(_) => {
                                                set_error_message.set("Failed to parse response".to_string());
                                            }
                                        }
                                    } else {
                                        match response.text().await {
                                            Ok(error_text) => {
                                                set_error_message.set(format!("Login failed: {}", error_text));
                                            }
                                            Err(_) => {
                                                set_error_message.set("Login failed: Unknown error".to_string());
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    set_error_message.set(format!("Network error: {}", e));
                                }
                            }
                            set_is_sent.set(false);
                        });
                    }
                }>
                    <div style="margin-bottom: 20px;">
                        <label style="
                            display: block;
                            margin-bottom: 8px;
                            font-weight: 500;
                            color: #333;
                            font-size: 0.95em;
                        ">"Username"</label>
                        <input 
                            type="text"
                            placeholder="Enter your username"
                            on:input:target=move |ev| {
                                set_username.set(ev.target().value());
                            }
                            prop:value=move || username.get()
                            style="
                                width: 100%;
                                padding: 16px 20px;
                                border: 2px solid #e9ecef;
                                border-radius: 12px;
                                font-size: 1em;
                                outline: none;
                                transition: all 0.3s ease;
                                background-color: #f8f9fa;
                                box-sizing: border-box;
                            "
                            disabled=move || is_sent.get()
                        />
                    </div>

                    <div style="margin-bottom: 30px;">
                        <label style="
                            display: block;
                            margin-bottom: 8px;
                            font-weight: 500;
                            color: #333;
                            font-size: 0.95em;
                        ">"Password"</label>
                        <input 
                            type="password"
                            placeholder="Enter your password"
                            on:input:target=move |ev| {
                                set_pwd.set(ev.target().value());
                            }
                            prop:value=move || pwd.get()
                            style="
                                width: 100%;
                                padding: 16px 20px;
                                border: 2px solid #e9ecef;
                                border-radius: 12px;
                                font-size: 1em;
                                outline: none;
                                transition: all 0.3s ease;
                                background-color: #f8f9fa;
                                box-sizing: border-box;
                            "
                            disabled=move || is_sent.get()
                        />
                    </div>

                    <button 
                        type="submit" 
                        disabled=move || is_sent.get() || pwd.get().trim().is_empty() || username.get().trim().is_empty()
                        style="
                            width: 100%;
                            background: linear-gradient(135deg, #667eea, #764ba2);
                            color: white;
                            border: none;
                            padding: 16px 24px;
                            border-radius: 12px;
                            cursor: pointer;
                            font-size: 1.1em;
                            font-weight: 600;
                            transition: all 0.3s ease;
                            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
                            margin-bottom: 20px;
                        "
                    >
                        {move || if is_sent.get() { "🔄 Signing in..." } else { "🚀 Sign In" }}
                    </button>
                </form>

                // Register Link
                <div style="
                    text-align: center;
                    padding-top: 20px;
                    border-top: 1px solid #e9ecef;
                ">
                    <p style="
                        margin: 0 0 10px 0;
                        color: #666;
                        font-size: 0.95em;
                    ">"Don't have an account?"</p>
                    <a href="/register" style="
                        color: #667eea;
                        text-decoration: none;
                        font-weight: 500;
                        font-size: 1em;
                        transition: all 0.3s ease;
                    ">
                        "Create account →"
                    </a>
                </div>
            </div>
        </div>  
    }
} 