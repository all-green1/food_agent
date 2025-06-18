use leptos::*;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
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
            // Main Content Card
            <div style="
                background-color: rgba(255, 255, 255, 0.95);
                backdrop-filter: blur(10px);
                border-radius: 20px;
                padding: 60px 40px;
                box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
                width: 100%;
                max-width: 600px;
                margin: 20px;
                text-align: center;
            ">
                // Header Section
                <div style="margin-bottom: 40px;">
                    <div style="
                        font-size: 4em;
                        margin-bottom: 20px;
                        animation: bounce 2s infinite;
                    ">"🍎"</div>
                    <h1 style="
                        margin: 0 0 15px 0;
                        font-size: 3em;
                        font-weight: 700;
                        background: linear-gradient(135deg, #667eea, #764ba2);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        background-clip: text;
                        line-height: 1.2;
                    ">"Food Agent"</h1>
                    <p style="
                        margin: 0;
                        color: #666;
                        font-size: 1.3em;
                        font-weight: 400;
                        line-height: 1.5;
                    ">"Reduce food waste with AI assistance"</p>
                </div>

                // Features Section
                <div style="
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                    gap: 20px;
                    margin: 40px 0;
                ">
                    <div style="
                        background: rgba(102, 126, 234, 0.1);
                        padding: 25px 20px;
                        border-radius: 15px;
                        border: 2px solid rgba(102, 126, 234, 0.2);
                    ">
                        <div style="font-size: 2.5em; margin-bottom: 10px;">"🤖"</div>
                        <h3 style="
                            margin: 0 0 10px 0;
                            color: #333;
                            font-size: 1.2em;
                            font-weight: 600;
                        ">"AI-Powered Chat"</h3>
                        <p style="
                            margin: 0;
                            color: #666;
                            font-size: 0.95em;
                            line-height: 1.4;
                        ">"Intelligent food management through natural conversation"</p>
                    </div>

                    <div style="
                        background: rgba(118, 75, 162, 0.1);
                        padding: 25px 20px;
                        border-radius: 15px;
                        border: 2px solid rgba(118, 75, 162, 0.2);
                    ">
                        <div style="font-size: 2.5em; margin-bottom: 10px;">"📊"</div>
                        <h3 style="
                            margin: 0 0 10px 0;
                            color: #333;
                            font-size: 1.2em;
                            font-weight: 600;
                        ">"Smart Tracking"</h3>
                        <p style="
                            margin: 0;
                            color: #666;
                            font-size: 0.95em;
                            line-height: 1.4;
                        ">"Track expiry dates and optimize your food inventory"</p>
                    </div>
                </div>

                // Call to Action Section
                <div style="margin-top: 50px;">
                    <p style="
                        margin: 0 0 30px 0;
                        color: #555;
                        font-size: 1.1em;
                        font-weight: 500;
                    ">"Ready to start managing your food smarter?"</p>
                    
                    <div style="
                        display: flex;
                        gap: 20px;
                        justify-content: center;
                        flex-wrap: wrap;
                    ">
                        <div style="
                            background: linear-gradient(135deg, #667eea, #764ba2);
                            border-radius: 15px;
                            padding: 3px;
                            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
                            transition: all 0.3s ease;
                        ">
                            <div style="
                                background: white;
                                border-radius: 12px;
                                padding: 16px 32px;
                                transition: all 0.3s ease;
                                text-decoration: none;
                                font-weight: 600;
                                font-size: 1.1em;
                                color: #667eea;
                            ">
                                <A href="/login">
                                    "🚀 Login"
                                </A>
                            </div>
                        </div>

                        <div style="
                            background: linear-gradient(135deg, #667eea, #764ba2);
                            border-radius: 15px;
                            padding: 16px 32px;
                            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
                            transition: all 0.3s ease;
                            color: white;
                            text-decoration: none;
                            font-weight: 600;
                            font-size: 1.1em;
                        ">
                            <A href="/register">
                                "✨ Sign Up"
                            </A>
                        </div>
                    </div>
                </div>

                // Footer Section
                <div style="
                    margin-top: 50px;
                    padding-top: 30px;
                    border-top: 1px solid #e9ecef;
                ">
                    <p style="
                        margin: 0;
                        color: #999;
                        font-size: 0.9em;
                    ">"Join thousands of users reducing food waste with AI"</p>
                </div>
            </div>
        </div>

        // Add CSS animations
        <style>
            "@keyframes bounce {
                0%, 20%, 50%, 80%, 100% {
                    transform: translateY(0);
                }
                40% {
                    transform: translateY(-10px);
                }
                60% {
                    transform: translateY(-5px);
                }
            }"
        </style>
    }
} 