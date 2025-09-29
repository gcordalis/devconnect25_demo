use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::ConnectInfo,
    http::Request,
    middleware::{self, Next},
    response::Html,
    routing::get,
    Json, Router,
};
use lazy_static::lazy_static;
use leptos::*;
use tower_http::trace::TraceLayer;

use hyper::StatusCode;
use tokio::net::TcpListener;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use axum::extract::FromRequest;
use hyper::header;

use tracing::info;

pub const DEFAULT_FIXTURE_PORT: u16 = 3000;

fn get_local_ip() -> String {
    if let Ok(ip) = local_ip_address::local_ip() {
        if !ip.is_loopback() {
            return ip.to_string();
        }
    }
    "localhost".to_string()
}

// Global log storage that persists across all TLS connections
lazy_static! {
    static ref GLOBAL_LOGS: Arc<RwLock<Vec<LogEntry>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

// Removed AppState struct - not needed for simple HTTP server

// Helper function to add logs to global storage
fn add_to_global_log(message: String) {
    let entry = LogEntry {
        timestamp: Utc::now(),
        message: message.clone(),
    };

    let mut logs = GLOBAL_LOGS.write().unwrap();
    logs.push(entry);

    // Keep only the last 15 entries to avoid unbounded memory growth
    if logs.len() > 15 {
        let len = logs.len();
        logs.drain(0..len - 15);
    }
}

fn app() -> Router {
    Router::new()
        .route("/", get(dashboard_handler))
        .route("/balances", get(balances_route))
        .layer(middleware::from_fn(access_log_middleware))
        .layer(TraceLayer::new_for_http())
}

/// Start the HTTP server
pub async fn serve() -> anyhow::Result<()> {
    let addr = std::env::var("ADDR").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .map(|port| port.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_FIXTURE_PORT);

    let listener = TcpListener::bind((addr.as_str(), port)).await?;
    info!("Starting HTTP server on {}:{}", addr, port);

    let app = app();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

// Removed old bind function - using axum::serve now

async fn access_log_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> axum::response::Response {
    // Only log requests to /balances
    if req.uri().path() == "/balances" {
        let ip = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        // Check authorization header
        let expected_token = "random_auth_token";
        let is_authorized = req
            .headers()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(|auth_token| {
                let token = auth_token.trim_start_matches("Bearer ");
                token == expected_token
            })
            .unwrap_or(false);

        let message = if is_authorized {
            format!("✅ Authorized access to /balances from {}", ip)
        } else {
            format!("❌ Unauthorized access attempt to /balances from {}", ip)
        };

        // Log to console and to our global in-memory log
        info!("{}", message);
        add_to_global_log(message);
    }

    next.run(req).await
}

/// parse the JSON data from the file content
fn get_json_value(filecontent: &str) -> Result<Json<Value>, StatusCode> {
    Ok(Json(serde_json::from_str(filecontent).map_err(|e| {
        eprintln!("Failed to parse JSON data: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?))
}

struct AuthenticatedUser;

impl<B> FromRequest<B> for AuthenticatedUser
where
    B: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(
        req: axum::extract::Request,
        _state: &B,
    ) -> Result<Self, Self::Rejection> {
        // Expected token (hardcoded for simplicity in the demo)
        let expected_token = "random_auth_token";

        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        if let Some(auth_token) = auth_header {
            let token = auth_token.trim_start_matches("Bearer ");
            if token == expected_token {
                return Ok(AuthenticatedUser);
            }
        }

        Err((StatusCode::UNAUTHORIZED, "Invalid or missing token"))
    }
}

async fn balances_route(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    _: AuthenticatedUser,
) -> Result<Json<Value>, StatusCode> {
    info!("Balances accessed from: {}", addr);
    get_bank_data()
}

fn get_bank_data() -> Result<Json<Value>, StatusCode> {
    get_json_value(include_str!("data/swissbankdata.json"))
}

async fn dashboard_handler() -> Html<String> {
    let local_ip = get_local_ip();
    let port = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_FIXTURE_PORT.to_string());
    let host = format!("{}:{}", local_ip, port);

    let app_html = leptos::ssr::render_to_string(move || view! { <App host=host /> });
    Html(app_html.to_string())
}

#[component]
pub fn App(#[prop(default = "localhost:3000".to_string())] host: String) -> impl IntoView {
    let data = get_bank_data().unwrap();
    let data = serde_json::to_string_pretty(&*data).unwrap();
    view! {
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta http-equiv="refresh" content="2"/>
                <title>"Swiss Bank Demo"</title>
                <style>
                    "
                    body {
                        font-family: Arial, sans-serif;
                        margin: 0;
                        padding: 20px;
                        background: white;
                        color: black;
                        font-size: 18px;
                        line-height: 1.6;
                    }
                    .container {
                        max-width: 1200px;
                        margin: 0 auto;
                    }
                    .header {
                        text-align: center;
                        margin-bottom: 40px;
                        padding: 30px;
                        border-bottom: 3px solid #333;
                    }
                    .header h1 {
                        font-size: 4rem;
                        margin: 0 0 20px 0;
                        color: #2c5aa0;
                    }
                    .header p {
                        font-size: 1.5rem;
                        margin: 0;
                        color: #666;
                    }
                    .section {
                        margin: 40px 0;
                        padding: 30px;
                        border: 2px solid #ddd;
                        border-radius: 10px;
                    }
                    .section h2 {
                        font-size: 2.5rem;
                        margin-top: 0;
                        color: #333;
                        border-bottom: 2px solid #333;
                        padding-bottom: 10px;
                    }
                    .balances-table {
                        width: 100%;
                        border-collapse: collapse;
                        font-size: 1.5rem;
                        margin: 20px 0;
                    }
                    .balances-table th,
                    .balances-table td {
                        padding: 15px;
                        text-align: left;
                        border-bottom: 2px solid #ddd;
                    }
                    .balances-table th {
                        background-color: #f8f9fa;
                        font-weight: bold;
                    }
                    .log-table {
                        width: 100%;
                        border-collapse: collapse;
                        font-size: 1.2rem;
                    }
                    .log-table th,
                    .log-table td {
                        padding: 12px;
                        text-align: left;
                        border-bottom: 1px solid #ddd;
                    }
                    .log-table th {
                        background-color: #f8f9fa;
                        font-weight: bold;
                        font-size: 1.3rem;
                    }
                    .log-table tbody tr:nth-child(even) {
                        background-color: #f8f9fa;
                    }
                    .status-authorized {
                        color: #28a745;
                        font-weight: bold;
                    }
                    .status-unauthorized {
                        color: #dc3545;
                        font-weight: bold;
                    }
                    .footer {
                        text-align: center;
                        margin-top: 40px;
                        padding: 20px;
                        background-color: #f8f9fa;
                        border-radius: 10px;
                        font-size: 1.3rem;
                    }
                    .footer code {
                        background: #e9ecef;
                        padding: 5px 10px;
                        border-radius: 5px;
                        font-family: monospace;
                    }
                    "
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>"Swiss Bank Demo"</h1>
                        <p>"This server holds EF's (fake) reserves. Only the EF has access."</p>
                    </div>

                    <div class="section">
                        <h2>"Bank Reserves"</h2>
                           <pre><code>{data}</code></pre>
                    </div>

                    <div class="section">
                        <h2>"Live Access Log"</h2>
                        <AccessLogTable/>
                    </div>

                    <div class="footer">
                        <p>"Try it yourself: " <code>{format!("http://{}/balances", host)}</code></p>
                    </div>
                </div>
            </body>
        </html>
    }
}

#[component]
pub fn AccessLogTable() -> impl IntoView {
    // Read logs from GLOBAL_LOGS during server rendering
    let logs = GLOBAL_LOGS.read().unwrap();
    let log_entries = logs.clone();

    view! {
        <div id="log-container">
            <table class="log-table">
                <thead>
                    <tr>
                        <th style="width: 120px;">"Time"</th>
                        <th>"Activity"</th>
                    </tr>
                </thead>
                <tbody id="log-entries">
                    {if log_entries.is_empty() {
                        view! {
                            <tr>
                                <td colspan="2" style="text-align: center; font-style: italic;">"Waiting for access attempts..."</td>
                            </tr>
                        }.into_view()
                    } else {
                        log_entries.into_iter().rev().map(|entry| {
                            let time_str = entry.timestamp.format("%H:%M:%S").to_string();
                            let status_class = if entry.message.contains("✅") {
                                "status-authorized"
                            } else if entry.message.contains("❌") {
                                "status-unauthorized"
                            } else {
                                ""
                            };

                            view! {
                                <tr>
                                    <td>{time_str}</td>
                                    <td class={status_class}>{entry.message}</td>
                                </tr>
                            }
                        }).collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}
