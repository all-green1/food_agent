use leptos::*;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

use crate::types::{AuthContext, RegisterRequest, RegisterResponse};

#[component]
pub fn SignUp() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (pwd, set_pwd) = signal(String::new());
    let (confirm_pwd, set_confirm_pwd) = signal(String::new());
    let (is_signed_up, set_is_signed_up) = signal::<Option<RegisterResponse>>(None);
    let (is_sent, set_is_sent) = signal(false);
    let (error_message, set_error_message) = signal(String::new());
    let auth_context = expect_context::<RwSignal<AuthContext>>();
    let navigate = use_navigate();

    // Computed signal to check if passwords match
    let passwords_match = move || {
        let password = pwd.get();
        let confirm = confirm_pwd.get();
        !password.is_empty() && !confirm.is_empty() && password == confirm
    };

    // Computed signal to check if form is valid
    let form_is_valid = move || {
        !username.get().trim().is_empty() && 
        !email.get().trim().is_empty() && 
        !pwd.get().trim().is_empty() && 
        !confirm_pwd.get().trim().is_empty() &&
        passwords_match()
    };

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
            // Signup Card
            <div style="
                background-color: rgba(255, 255, 255, 0.95);
                backdrop-filter: blur(10px);
                border-radius: 20px;
                padding: 40px;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
                width: 100%;
                max-width: 450px;
                margin: 20px;
                max-height: 90vh;
                overflow-y: auto;
            ">
                // Header
                <div style="text-align: center; margin-bottom: 30px;">
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
                    ">"Join Food Agent"</h1>
                    <p style="
                        margin: 0;
                        color: #666;
                        font-size: 1.1em;
                    ">"Create your account to get started"</p>
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
                                        "Welcome, "{username}"! 🎉"
                                    </div>
                                    <div style="font-size: 0.9em; opacity: 0.9; margin-top: 5px;">
                                        "Account created successfully"
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
                
                // Signup Form
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    let password = pwd.get();
                    let user_name = username.get();
                    let user_email = email.get();
                    let confirm_password = confirm_pwd.get();
                    
                    if !user_name.is_empty() && !user_email.is_empty() && !password.is_empty() && !confirm_password.is_empty() {
                        if password != confirm_password {
                            set_error_message.set("Passwords do not match".to_string());
                            return;
                        }
                        
                        set_is_sent.set(true);
                        set_error_message.set(String::new()); // Clear previous errors
                        
                        // Clone navigate before async move
                        let nav = navigate.clone();
                        
                        spawn_local(async move {
                            let register_details = RegisterRequest {
                                email: user_email.clone(),
                                username: user_name.clone(),
                                password: password.clone(),
                            };
                            match Request::post("http://localhost:8000/auth/register")
                                .json(&register_details)
                                .unwrap()
                                .send()
                                .await
                            {
                                Ok(response) => {
                                    if response.ok() {  // Check if status is 200-299
                                        match response.json::<RegisterResponse>().await {
                                            Ok(register_response) => {
                                                set_is_signed_up.set(Some(register_response.clone()));
                                                let new_auth_context = AuthContext {
                                                    token: Some(register_response.access_token.clone()),
                                                    user_id: Some(register_response.user_id),
                                                    username: Some(register_response.username.clone()),
                                                    is_authenticated: true,
                                                };
                                                
                                                // Save to localStorage before setting the context
                                                new_auth_context.save_to_storage();
                                                auth_context.set(new_auth_context);
                                                
                                                nav("/chat", Default::default());
                                            }
                                            Err(_) => {
                                                set_error_message.set("Failed to parse response".to_string());
                                            }
                                        }
                                    } else {  // Handle error status codes (400, 500, etc.)
                                        match response.text().await {
                                            Ok(error_text) => {
                                                set_error_message.set(format!("Registration failed: {}", error_text));
                                            }
                                            Err(_) => {
                                                set_error_message.set("Registration failed: Unknown error".to_string());
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
                            placeholder="Choose a username"
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

                    <div style="margin-bottom: 20px;">
                        <label style="
                            display: block;
                            margin-bottom: 8px;
                            font-weight: 500;
                            color: #333;
                            font-size: 0.95em;
                        ">"Email"</label>
                        <input 
                            type="email"
                            placeholder="Enter your email address"
                            on:input:target=move |ev| {
                                set_email.set(ev.target().value());
                            }
                            prop:value=move || email.get()
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

                    <div style="margin-bottom: 20px;">
                        <label style="
                            display: block;
                            margin-bottom: 8px;
                            font-weight: 500;
                            color: #333;
                            font-size: 0.95em;
                        ">"Password"</label>
                        <input 
                            type="password"
                            placeholder="Create a strong password"
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

                    <div style="margin-bottom: 20px;">
                        <label style="
                            display: block;
                            margin-bottom: 8px;
                            font-weight: 500;
                            color: #333;
                            font-size: 0.95em;
                        ">"Confirm Password"</label>
                        <input 
                            type="password"
                            placeholder="Confirm your password"
                            on:input:target=move |ev| {
                                set_confirm_pwd.set(ev.target().value());
                            }
                            prop:value=move || confirm_pwd.get()
                            style={move || format!("
                                width: 100%;
                                padding: 16px 20px;
                                border: 2px solid {};
                                border-radius: 12px;
                                font-size: 1em;
                                outline: none;
                                transition: all 0.3s ease;
                                background-color: #f8f9fa;
                                box-sizing: border-box;
                            ", if confirm_pwd.get().is_empty() {
                                "#e9ecef"
                            } else if passwords_match() {
                                "#4CAF50"
                            } else {
                                "#f44336"
                            })}
                            disabled=move || is_sent.get()
                        />
                        {move || {
                            if !confirm_pwd.get().is_empty() && !pwd.get().is_empty() {
                                if passwords_match() {
                                    Some(view! {
                                        <div style="
                                            color: #4CAF50;
                                            font-size: 0.85em;
                                            margin-top: 5px;
                                            font-weight: 500;
                                        ">
                                            "✓ Passwords match"
                                        </div>
                                    })
                                } else {
                                    Some(view! {
                                        <div style="
                                            color: #f44336;
                                            font-size: 0.85em;
                                            margin-top: 5px;
                                            font-weight: 500;
                                        ">
                                            "✗ Passwords do not match"
                                        </div>
                                    })
                                }
                            } else {
                                None
                            }
                        }}
                    </div>

                    <button 
                        type="submit" 
                        disabled=move || is_sent.get() || !form_is_valid()
                        style={move || format!("
                            width: 100%;
                            background: {};
                            color: white;
                            border: none;
                            padding: 16px 24px;
                            border-radius: 12px;
                            cursor: {};
                            font-size: 1.1em;
                            font-weight: 600;
                            transition: all 0.3s ease;
                            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
                            margin-bottom: 20px;
                            opacity: {};
                        ", 
                        if form_is_valid() && !is_sent.get() {
                            "linear-gradient(135deg, #667eea, #764ba2)"
                        } else {
                            "#ccc"
                        },
                        if form_is_valid() && !is_sent.get() {
                            "pointer"
                        } else {
                            "not-allowed"
                        },
                        if form_is_valid() && !is_sent.get() {
                            "1"
                        } else {
                            "0.6"
                        })}
                    >
                        {move || if is_sent.get() { 
                            "🔄 Creating Account..." 
                        } else if form_is_valid() {
                            "🚀 Create Account"
                        } else {
                            "Complete All Fields"
                        }}
                    </button>
                </form>

                // Login Link
                <div style="
                    text-align: center;
                    padding-top: 20px;
                    border-top: 1px solid #e9ecef;
                ">
                    <p style="
                        margin: 0 0 10px 0;
                        color: #666;
                        font-size: 0.95em;
                    ">"Already have an account?"</p>
                    <a href="/login" style="
                        color: #667eea;
                        text-decoration: none;
                        font-weight: 500;
                        font-size: 1em;
                        transition: all 0.3s ease;
                    ">
                        "Sign in →"
                    </a>
                </div>
            </div>
        </div>
    }
} 