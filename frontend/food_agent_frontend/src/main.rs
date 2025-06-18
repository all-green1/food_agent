use leptos::*;
use leptos_router::{components::{Router, Routes, Route}, path};
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use leptos::prelude::*;

mod chat;
mod login;
mod home;
mod signup;
mod types;

use chat::ChatPage;
use login::LoginPage;
use home::HomePage;
use signup::SignUp;
use types::AuthContext;

// App initialization function
async fn initialize_app() {
    console_error_panic_hook::set_once();
    
    // Additional app setup can go here
    leptos::logging::log!("Food Agent Chat App initializing...");
    
    // Mount the main app component
    mount_to_body(|| view! { <App/> })
}

fn main() {
    spawn_local(initialize_app());
}

#[component]
pub fn App() -> impl IntoView {
    // Load auth context from localStorage on startup
    let initial_auth = AuthContext::load_from_storage();
    let auth_context = RwSignal::new(initial_auth);
    provide_context(auth_context);
    
    view! {
        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/login") view=LoginPage />
                <Route path=path!("/register") view=SignUp />
                <Route path=path!("/chat") view=ChatPage />
            </Routes>
        </Router>
    }
}

#[component]
pub fn AppHeader() -> impl IntoView {
    view! {
        <header style="padding: 10px; background-color:rgb(255, 255, 255); margin-bottom: 20px;">
            <h1>"Food Agent - AI Food Management"</h1>
            <p>"Manage your food inventory with AI assistance"</p>
        </header>
    }
}

#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer style="margin-top: 40px; text-align: center; color: #666; font-size: 0.9em;">
            <p>"Food Agent v1.0 - Powered by Rust + Leptos"</p>
        </footer>
    }
}
