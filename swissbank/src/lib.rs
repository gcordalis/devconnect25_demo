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
use dioxus::prelude::*;
use lazy_static::lazy_static;
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
const AUTH_TOKEN: &str = "random_auth_token";
const DASHBOARD_CSS: &str = include_str!("dashboard.css");

fn get_local_ip() -> String {
    if let Ok(ip) = local_ip_address::local_ip() {
        if !ip.is_loopback() {
            return ip.to_string();
        }
    }
    "localhost".to_string()
}

lazy_static! {
    static ref GLOBAL_LOGS: Arc<RwLock<Vec<LogEntry>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

// Helper function to add logs to global storage
fn add_to_global_log(message: String) {
    let entry = LogEntry {
        timestamp: Utc::now(),
        message: message,
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
        .route("/", get(home_handler))
        .route("/dashboard", get(dashboard_handler))
        .route("/balances", get(balances_route))
        .route("/logs", get(logs_endpoint))
        .route("/logs-html", get(logs_html_endpoint))
        .layer(middleware::from_fn(access_log_middleware))
        .layer(TraceLayer::new_for_http())
}

async fn logs_html_endpoint() -> Html<String> {
    let logs = GLOBAL_LOGS.read().unwrap();

    let html = if logs.is_empty() {
        String::from(
            r#"<tr><td colspan="2" style="text-align: center; font-style: italic;">Waiting for access attempts...</td></tr>"#,
        )
    } else {
        logs.iter()
            .rev()
            .map(|entry| {
                let time_str = entry.timestamp.format("%H:%M:%S").to_string();
                let status_class = if entry.message.contains("✅") {
                    "status-authorized"
                } else if entry.message.contains("❌") {
                    "status-unauthorized"
                } else {
                    ""
                };
                format!(
                    r#"<tr><td>{}</td><td class="{}">{}</td></tr>"#,
                    time_str, status_class, entry.message
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(html)
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
        let is_authorized = req
            .headers()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(|auth_token| {
                let token = auth_token.trim_start_matches("Bearer ");
                token == AUTH_TOKEN
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

struct AuthenticatedUser;

impl<S> FromRequest<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(
        req: axum::extract::Request,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        if let Some(auth_token) = auth_header {
            let token = auth_token.trim_start_matches("Bearer ");
            if token == AUTH_TOKEN {
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
    Ok(Json(
        serde_json::from_str(include_str!("data/swissbankdata.json")).map_err(|e| {
            eprintln!("Failed to parse JSON data: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    ))
}

async fn logs_endpoint() -> Json<Vec<LogEntry>> {
    let logs = GLOBAL_LOGS.read().unwrap();
    Json(logs.clone())
}

async fn home_handler() -> Html<String> {
    let mut vdom = VirtualDom::new(HomePage);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    Html(html)
}

async fn dashboard_handler() -> Html<String> {
    let local_ip = get_local_ip();
    let port = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_FIXTURE_PORT.to_string());
    let host = format!("{}:{}", local_ip, port);

    let mut vdom = VirtualDom::new_with_props(App, AppProps { host });
    vdom.rebuild_in_place();
    let app_html = dioxus_ssr::render(&vdom);
    Html(app_html)
}

#[derive(Props, Clone, PartialEq)]
pub struct AppProps {
    host: String,
}

fn redact_json(json_str: &str) -> String {
    json_str
        .chars()
        .map(|c| match c {
            '0'..='9' => '█',
            '_' => '█',
            'a'..='z' | 'A'..='Z' => '█',
            _ => c,
        })
        .collect()
}

#[component]
pub fn HomePage() -> Element {
    rsx! {
        head {
            meta { charset: "utf-8" }
            meta { name: "viewport", content: "width=device-width, initial-scale=1" }
            title { "Swiss Bank Demo" }
            style { dangerous_inner_html: DASHBOARD_CSS }
        }
        body { class: "home-body",
            div { class: "home-container",
                h1 { "TLSNotary Swiss Bank Demo" }
                div { class: "description",
                    p { "This is a demonstration of TLSNotary technology using a simulated Swiss bank scenario. The bank's reserves are protected behind an authentication token, demonstrating how TLSNotary can verify private API data without revealing credentials." }
                    p { "Use the dashboard to monitor access attempts in real-time and see how authenticated requests reveal the bank data." }
                }
                div { class: "links",
                    a { class: "link-item", href: "/dashboard",
                        div { class: "link-title", "Dashboard" }
                        div { class: "link-desc", "View the live access log and bank reserves" }
                    }
                    a { class: "link-item", href: "/balances",
                        div { class: "link-title", "Balances" }
                        div { class: "link-desc", "Access bank balances (requires authentication)" }
                    }
                    a { class: "link-item", href: "/logs",
                        div { class: "link-title", "Logs" }
                        div { class: "link-desc", "View access logs in JSON format" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn App(props: AppProps) -> Element {
    let data = get_bank_data().unwrap();
    let data_str = serde_json::to_string_pretty(&*data).unwrap();
    let redacted = redact_json(&data_str);

    let data_escaped = data_str
        .replace("\\", "\\\\")
        .replace("`", "\\`")
        .replace("$", "\\$");
    let redacted_escaped = redacted
        .replace("\\", "\\\\")
        .replace("`", "\\`")
        .replace("$", "\\$");

    rsx! {
        head {
            meta { charset: "utf-8" }
            meta { name: "viewport", content: "width=device-width, initial-scale=1" }
            title { "Swiss Bank Demo" }
            script { src: "https://unpkg.com/htmx.org@1.9.10" }
            style { dangerous_inner_html: DASHBOARD_CSS }
        }
        body {
            div { class: "container",
                    div { class: "header",
                        h1 { "TLSNotary Swiss Bank Demo" }
                        p { "This demo server holds the EF's (fake) cash reserves." }
                        p { "Only the EF has the authentication token to access the data" }

                    }

                    div { class: "content-grid",
                        div { class: "section",
                            h2 { "Bank Reserves" }
                            pre {
                                code { id: "bank-data", "{redacted}" }
                            }
                            p { style: "font-size: 0.9em; font-style: italic; color: #666; margin-top: 10px;", "(Hold 'S' to reveal data)" }
                        }

                        div { class: "section",
                            h2 { "Live Access Log" }
                            table { class: "log-table",
                                thead {
                                    tr {
                                        th { style: "width: 120px;", "Time" }
                                        th { "Activity" }
                                    }
                                }
                                tbody {
                                    "hx-get": "/logs-html",
                                    "hx-trigger": "load, every 2s",
                                    "hx-swap": "innerHTML"
                                }
                            }
                        }
                    }
            }
            script { dangerous_inner_html: "
                (function() {{
                    const realData = `{data_escaped}`;
                    const redactedData = `{redacted_escaped}`;

                    document.addEventListener('keydown', function(event) {{
                        if (event.key === 's' || event.key === 'S') {{
                            const codeElement = document.getElementById('bank-data');
                            if (codeElement) {{
                                codeElement.textContent = realData;
                            }}
                        }}
                    }});

                    document.addEventListener('keyup', function(event) {{
                        if (event.key === 's' || event.key === 'S') {{
                            const codeElement = document.getElementById('bank-data');
                            if (codeElement) {{
                                codeElement.textContent = redactedData;
                            }}
                        }}
                    }});
                }})();
            " }
        }
    }
}
