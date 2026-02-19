use crate::state::State;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tracing::{debug, error, info};

pub(crate) fn build_stats_json(state: &State) -> serde_json::Value {
    let uptime = state.metrics.start_time.elapsed();
    serde_json::json!({
        "uptime_secs": uptime.as_secs(),
        "messages": state.metrics.message_count.load(Ordering::Relaxed),
        "active_sessions": state.session_mgr.sessions.len(),
        "allowed_senders": state.allowed_ids.len(),
        "total_cost_usd": state.total_cost_usd(),
        "model": state.config.model,
        "version": env!("CARGO_PKG_VERSION"),
    })
}

pub(crate) async fn run_stats_server(listener: TcpListener, state: Arc<State>) {
    info!(addr = %listener.local_addr().unwrap(), "Stats server listening");
    loop {
        let (mut stream, addr) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Stats accept error: {e}");
                continue;
            }
        };
        debug!(peer = %addr, "Stats connection");
        let state = state.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
            let body = build_stats_json(&state).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::tests::test_state_with;
    use crate::traits::{MockClaudeRunner, MockSignalApi};

    #[test]
    fn test_build_stats_json_initially_zero() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        let json = build_stats_json(&state);

        assert_eq!(json["messages"], 0);
        assert_eq!(json["active_sessions"], 0);
        assert_eq!(json["total_cost_usd"], 0.0);
    }

    #[test]
    fn test_stats_includes_all_fields() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        let json = build_stats_json(&state);

        // All expected fields must be present
        assert!(json.get("uptime_secs").is_some(), "missing uptime_secs");
        assert!(json.get("messages").is_some(), "missing messages");
        assert!(
            json.get("active_sessions").is_some(),
            "missing active_sessions"
        );
        assert!(
            json.get("allowed_senders").is_some(),
            "missing allowed_senders"
        );
        assert!(
            json.get("total_cost_usd").is_some(),
            "missing total_cost_usd"
        );
        assert!(json.get("model").is_some(), "missing model");
        assert!(json.get("version").is_some(), "missing version");
    }

    #[test]
    fn test_stats_json_format() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        let json = build_stats_json(&state);

        // Must be a valid JSON object (not array, not scalar)
        assert!(json.is_object());

        // Numeric fields are numbers
        assert!(json["uptime_secs"].is_u64());
        assert!(json["messages"].is_u64());
        assert!(json["active_sessions"].is_u64());
        assert!(json["allowed_senders"].is_u64());
        assert!(json["total_cost_usd"].is_f64());

        // String fields are strings
        assert!(json["model"].is_string());
        assert!(json["version"].is_string());

        // Model reflects the test state
        assert_eq!(json["model"], "sonnet");
    }

    #[test]
    fn test_stats_reflects_state_mutations() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());

        // Initial state
        let json = build_stats_json(&state);
        assert_eq!(json["messages"], 0);
        assert_eq!(json["total_cost_usd"], 0.0);
        assert_eq!(json["active_sessions"], 0);

        // Mutate: add messages
        state.metrics.message_count.fetch_add(42, Ordering::Relaxed);
        let json = build_stats_json(&state);
        assert_eq!(json["messages"], 42);

        // Mutate: add cost
        state.add_cost(1.23);
        let json = build_stats_json(&state);
        assert!((json["total_cost_usd"].as_f64().unwrap() - 1.23).abs() < 0.001);

        // Mutate: add a session
        state.session_mgr.sessions.insert(
            "+someone".to_string(),
            crate::state::SenderState {
                session_id: "sess-1".to_string(),
                model: "sonnet".to_string(),
                lock: Arc::new(tokio::sync::Mutex::new(())),
                last_activity: std::time::Instant::now(),
            },
        );
        let json = build_stats_json(&state);
        assert_eq!(json["active_sessions"], 1);
    }

    #[tokio::test]
    async fn test_stats_server_responds_with_json() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        state.metrics.message_count.store(7, Ordering::Relaxed);
        state.add_cost(0.42);

        let state = Arc::new(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn the stats server
        let server_state = state.clone();
        tokio::spawn(run_stats_server(listener, server_state));

        // Connect and send a minimal HTTP request
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        tokio::io::AsyncWriteExt::write_all(
            &mut stream,
            b"GET /stats HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await
        .unwrap();

        // Read response
        let mut response = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut response)
            .await
            .unwrap();
        let response_str = String::from_utf8(response).unwrap();

        // Verify HTTP response structure
        assert!(
            response_str.starts_with("HTTP/1.1 200 OK"),
            "Expected 200 OK, got: {}",
            &response_str[..40.min(response_str.len())]
        );
        assert!(response_str.contains("Content-Type: application/json"));

        // Extract JSON body (after \r\n\r\n)
        let body = response_str
            .split("\r\n\r\n")
            .nth(1)
            .expect("no body in response");
        let json: serde_json::Value = serde_json::from_str(body).expect("invalid JSON in body");

        assert_eq!(json["messages"], 7);
        assert!((json["total_cost_usd"].as_f64().unwrap() - 0.42).abs() < 0.001);
        assert_eq!(json["model"], "sonnet");
        assert!(json["version"].is_string());
    }
}
