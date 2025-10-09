use ::server::{config::Config, prover::prover, run_ws_server, verifier::verifier};
use async_tungstenite::{tokio::connect_async_with_config, tungstenite::protocol::WebSocketConfig};
use eyre::eyre;
use rstest::*;
use std::sync::Once;
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid;
use ws_stream_tungstenite::WsStream;
const TRACING_FILTER: &str = "INFO";
const SERVER_START_DELAY: Duration = Duration::from_millis(500);
const TEST_TIMEOUT: Duration = Duration::from_secs(60);

static INIT: Once = Once::new();

fn init_tracing() {
    let _ = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| TRACING_FILTER.into()))
        .with(tracing_subscriber::fmt::layer())
        .try_init();
}

fn ensure_server_started() {
    INIT.call_once(|| {
        init_tracing();

        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config = Config::default();
                run_ws_server(&config)
                    .await
                    .expect("Server should start successfully")
            });
        });

        // Give the server time to start
        std::thread::sleep(SERVER_START_DELAY);
    });
}

#[fixture]
fn server() {
    ensure_server_started();
}

/// Create a WebSocket connection request with standard headers
/// This eliminates duplication of WebSocket request creation across modules
pub fn create_websocket_request(host: &str, port: u16, path: &str) -> http::Request<()> {
    http::Request::builder()
        .uri(format!("ws://{host}:{port}{path}"))
        .header("Host", host)
        .header("Sec-WebSocket-Key", uuid::Uuid::new_v4().to_string())
        .header("Sec-WebSocket-Version", "13")
        .header("Connection", "Upgrade")
        .header("Upgrade", "Websocket")
        .body(())
        .unwrap()
}

#[rstest]
#[tokio::test]
async fn test_prover_verifier_integration(_server: ()) {
    let config = Config::default();
    let result = timeout(TEST_TIMEOUT, async {
        info!("Connecting to server as verifier...");
        let request = create_websocket_request(&config.ws_host, config.ws_port, "/prove");
        let (ws_stream, _) = connect_async_with_config(request, Some(WebSocketConfig::default()))
            .await
            .map_err(|e| eyre!("Failed to connect to server: {}", e))?;
        let server_ws_socket = WsStream::new(ws_stream);
        info!("WebSocket connection established with server!");
        verifier(server_ws_socket, &config.server_domain()).await?;
        info!("Verification completed successfully!");
        Ok::<(), eyre::ErrReport>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Integration test passed: Prover-Verifier communication successful")
        }
        Ok(Err(e)) => panic!("❌ Test failed: {}", e),
        Err(_) => panic!("❌ Test timed out after {:?}", TEST_TIMEOUT),
    }
}

#[rstest]
#[tokio::test]
async fn test_verifier_prover_integration(_server: ()) {
    let config = Config::default();
    let result = timeout(TEST_TIMEOUT, async {
        info!("Connecting to server as prover...");
        let request = create_websocket_request(&config.ws_host, config.ws_port, "/verify");
        let (ws_stream, _) = connect_async_with_config(request, Some(WebSocketConfig::default()))
            .await
            .map_err(|e| eyre!("Failed to connect to server: {}", e))?;
        let server_ws_socket = WsStream::new(ws_stream);
        info!("WebSocket connection established with server!");
        prover(server_ws_socket, &config.server_uri).await?;
        info!("Proving completed successfully!");
        Ok::<(), eyre::ErrReport>(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("✅ Integration test passed: Verifier-Prover communication successful")
        }
        Ok(Err(e)) => panic!("❌ Test failed: {}", e),
        Err(_) => panic!("❌ Test timed out after {:?}", TEST_TIMEOUT),
    }
}
