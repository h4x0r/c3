use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::error::AppError;
use crate::helpers::{hash_message, split_message};
use crate::signal::AttachmentInfo;
use crate::traits::{ClaudeRunner, SignalApi};

pub(crate) struct PendingSender {
    pub(crate) name: String,
    pub(crate) short_id: u64,
}

pub(crate) struct SenderState {
    pub(crate) session_id: String,
    pub(crate) model: String,
    pub(crate) lock: Arc<Mutex<()>>,
    pub(crate) last_activity: Instant,
}

pub(crate) struct TokenBucket {
    pub(crate) tokens: f64,
    pub(crate) last_refill: Instant,
    pub(crate) capacity: f64,
    pub(crate) rate_per_sec: f64,
}

impl TokenBucket {
    pub(crate) fn new(capacity: f64, rate_per_sec: f64) -> Self {
        Self {
            tokens: capacity,
            last_refill: Instant::now(),
            capacity,
            rate_per_sec,
        }
    }

    pub(crate) fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate_per_sec).min(self.capacity);
        self.last_refill = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Immutable configuration set at startup from CLI args.
pub(crate) struct Config {
    pub(crate) model: String,
    pub(crate) max_budget: f64,
    pub(crate) rate_limit_config: Option<(f64, f64)>,
    pub(crate) session_ttl: Option<Duration>,
    pub(crate) debounce_ms: u64,
    pub(crate) account: String,
    pub(crate) api_url: String,
    pub(crate) config_path: Option<String>,
}

/// Runtime metrics (atomic counters).
pub(crate) struct Metrics {
    pub(crate) start_time: Instant,
    pub(crate) message_count: AtomicU64,
    pub(crate) total_cost: AtomicU64, // stored as microdollars
}

/// Per-sender session tracking.
pub(crate) struct SessionManager {
    pub(crate) sessions: DashMap<String, SenderState>,
    pub(crate) truncated_sessions: DashMap<String, String>,
}

/// Debounce state for merging burst messages.
pub(crate) struct DebounceState {
    pub(crate) buffers: DashMap<String, (Vec<String>, Instant)>,
    pub(crate) active: DashMap<String, ()>,
}

pub(crate) struct State {
    pub(crate) config: Config,
    pub(crate) metrics: Metrics,
    pub(crate) session_mgr: SessionManager,
    pub(crate) debounce: DebounceState,
    pub(crate) allowed_ids: DashMap<String, ()>,
    pub(crate) pending_senders: DashMap<String, PendingSender>,
    pub(crate) pending_counter: AtomicU64,
    pub(crate) sent_hashes: Arc<DashMap<u64, ()>>,
    pub(crate) rate_limits: DashMap<String, TokenBucket>,
    pub(crate) signal_api: Box<dyn SignalApi>,
    pub(crate) claude_runner: Box<dyn ClaudeRunner>,
}

impl State {
    pub(crate) fn is_allowed(&self, sender: &str) -> bool {
        !sender.is_empty() && self.allowed_ids.contains_key(sender)
    }

    pub(crate) fn add_cost(&self, cost: f64) {
        let micros = (cost * 1_000_000.0) as u64;
        self.metrics.total_cost.fetch_add(micros, Ordering::Relaxed);
    }

    pub(crate) fn total_cost_usd(&self) -> f64 {
        self.metrics.total_cost.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    pub(crate) async fn send_message(&self, recipient: &str, message: &str) -> Result<(), AppError> {
        self.sent_hashes.insert(hash_message(message), ());
        self.signal_api.send_msg(recipient, message).await
    }

    pub(crate) async fn send_long_message(&self, recipient: &str, message: &str) -> Result<(), AppError> {
        let parts = split_message(message, crate::constants::MAX_SIGNAL_MSG_LEN);
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            self.send_message(recipient, part).await?;
        }
        Ok(())
    }

    pub(crate) async fn set_typing(&self, recipient: &str, typing: bool) -> Result<(), AppError> {
        self.signal_api.set_typing(recipient, typing).await
    }

    pub(crate) async fn download_attachment(&self, attachment: &AttachmentInfo) -> Result<PathBuf, AppError> {
        self.signal_api.download_attachment(attachment).await
    }

    /// Get or create a session for a sender. Returns (session_id, model, lock, is_new).
    pub(crate) fn get_or_create_session(&self, sender: &str) -> (String, String, Arc<Mutex<()>>, bool) {
        let is_new = !self.session_mgr.sessions.contains_key(sender);
        let mut entry = self
            .session_mgr
            .sessions
            .entry(sender.to_string())
            .or_insert_with(|| {
                let session_id = uuid::Uuid::new_v4().to_string();
                tracing::info!(sender = %sender, session_id = %session_id, "New session created");
                SenderState {
                    session_id,
                    model: self.config.model.clone(),
                    lock: Arc::new(Mutex::new(())),
                    last_activity: Instant::now(),
                }
            });
        entry.last_activity = Instant::now();
        (
            entry.session_id.clone(),
            entry.model.clone(),
            entry.lock.clone(),
            is_new,
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::traits::{MockClaudeRunner, MockSignalApi};

    pub(crate) fn test_state_with(signal: MockSignalApi, claude: MockClaudeRunner) -> State {
        State {
            config: Config {
                model: "sonnet".to_string(),
                max_budget: 5.0,
                rate_limit_config: None,
                session_ttl: None,
                debounce_ms: 0,
                account: "+1234567890".to_string(),
                api_url: "http://127.0.0.1:9999".to_string(),
                config_path: None,
            },
            metrics: Metrics {
                start_time: Instant::now(),
                message_count: AtomicU64::new(0),
                total_cost: AtomicU64::new(0),
            },
            session_mgr: SessionManager {
                sessions: DashMap::new(),
                truncated_sessions: DashMap::new(),
            },
            debounce: DebounceState {
                buffers: DashMap::new(),
                active: DashMap::new(),
            },
            allowed_ids: {
                let m = DashMap::new();
                m.insert("+1234567890".to_string(), ());
                m.insert("+allowed_user".to_string(), ());
                m
            },
            pending_senders: DashMap::new(),
            pending_counter: AtomicU64::new(0),
            sent_hashes: Arc::new(DashMap::new()),
            rate_limits: DashMap::new(),
            signal_api: Box::new(signal),
            claude_runner: Box::new(claude),
        }
    }

    #[test]
    fn test_token_bucket_basic() {
        let mut bucket = TokenBucket::new(3.0, 1.0);
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(2.0, 1000.0);
        assert!(bucket.try_consume());
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume());
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(bucket.try_consume());
    }

    #[test]
    fn test_rate_limit_blocks() {
        let mut bucket = TokenBucket::new(1.0, 0.0);
        assert!(bucket.try_consume());
        assert!(!bucket.try_consume());
        assert!(!bucket.try_consume());
        assert!(!bucket.try_consume());
    }

    #[test]
    fn test_session_expiry_logic() {
        let ttl = Duration::from_millis(50);
        let before = Instant::now();
        std::thread::sleep(Duration::from_millis(60));
        let elapsed = before.elapsed();
        assert!(elapsed > ttl, "elapsed time should exceed TTL");
    }

    #[test]
    fn test_state_is_allowed_true() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        assert!(state.is_allowed("+allowed_user"));
    }

    #[test]
    fn test_state_is_allowed_false() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        assert!(!state.is_allowed("+unknown_user"));
    }

    #[test]
    fn test_state_is_allowed_empty() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        assert!(!state.is_allowed(""));
    }

    #[test]
    fn test_get_or_create_session_new() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        let (session_id, model, _lock, is_new) = state.get_or_create_session("+fresh_user");
        assert!(is_new);
        assert!(!session_id.is_empty());
        assert_eq!(model, "sonnet");
    }

    #[test]
    fn test_get_or_create_session_existing_returns_same() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        let (id1, _, _, new1) = state.get_or_create_session("+repeat_user");
        assert!(new1);
        let (id2, _, _, new2) = state.get_or_create_session("+repeat_user");
        assert!(!new2);
        assert_eq!(id1, id2, "same sender should get same session");
    }

    #[test]
    fn test_add_cost_and_total_cost_usd() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        assert!((state.total_cost_usd() - 0.0).abs() < f64::EPSILON);
        state.add_cost(0.5);
        assert!((state.total_cost_usd() - 0.5).abs() < 0.001);
        state.add_cost(1.25);
        assert!((state.total_cost_usd() - 1.75).abs() < 0.001);
    }

    #[test]
    fn test_add_cost_fractional() {
        let state = test_state_with(MockSignalApi::new(), MockClaudeRunner::new());
        state.add_cost(0.000001);
        assert!(state.total_cost_usd() > 0.0);
        assert!(state.total_cost_usd() < 0.001);
    }

    #[tokio::test]
    async fn test_state_send_long_message_splits() {
        let mut signal = MockSignalApi::new();
        let call_count = Arc::new(AtomicU64::new(0));
        let count_clone = Arc::clone(&call_count);
        signal.expect_send_msg().returning(move |_, _| {
            count_clone.fetch_add(1, Ordering::Relaxed);
            Ok(())
        });

        let state = test_state_with(signal, MockClaudeRunner::new());
        let long_msg = "a".repeat(6000);
        let result = state.send_long_message("+allowed_user", &long_msg).await;
        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::Relaxed), 2);
    }
}
