/// Maximum Signal message length before splitting into multiple parts.
pub(crate) const MAX_SIGNAL_MSG_LEN: usize = 4000;

/// Response length threshold above which we check for truncation.
pub(crate) const TRUNCATION_THRESHOLD: usize = 3500;

/// Temp directory for downloaded attachments and isolated workdirs.
pub(crate) const TMP_DIR: &str = "/tmp/ccchat";

/// Default debounce window in milliseconds.
pub(crate) const DEFAULT_DEBOUNCE_MS: u64 = 3000;

/// Default port for signal-cli-api.
pub(crate) const DEFAULT_PORT: u16 = 8080;

/// Default Claude model.
pub(crate) const DEFAULT_MODEL: &str = "opus";

/// Default max budget per message in USD.
pub(crate) const DEFAULT_MAX_BUDGET: f64 = 5.0;

/// Budget for session summarization calls.
pub(crate) const SUMMARIZE_BUDGET: f64 = 0.05;

/// Max number of memory search results to return.
pub(crate) const MEMORY_SEARCH_LIMIT: usize = 5;

/// Max number of recent summaries to keep per sender.
pub(crate) const MAX_SUMMARIES: usize = 5;

/// Number of seconds in a day.
pub(crate) const SECS_PER_DAY: i64 = 86400;

/// Number of recent messages to capture when pinning.
pub(crate) const PIN_MESSAGE_COUNT: usize = 10;

/// Number of message pairs before triggering auto-summarization.
pub(crate) const AUTO_SUMMARIZE_THRESHOLD: u64 = 20;
