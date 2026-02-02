// Rate-limited LLM wrapper with global rate limiting
use adk_core::{Llm, LlmRequest, LlmResponseStream};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{OnceCell, Semaphore};
use tokio::time::{sleep, Duration};

/// Global rate limiter for all LLM calls
/// Ensures that only a limited number of requests are made per time window
static GLOBAL_RATE_LIMITER: OnceCell<Arc<Semaphore>> = OnceCell::const_new();

/// Initialize the global rate limiter
/// 
/// This should be called once when the application starts
/// to set up the global rate limiting semaphore.
/// 
/// # Arguments
/// * `max_concurrent` - Maximum number of concurrent requests allowed
/// 
/// # Safety
/// This function can be called multiple times safely - it will only
/// initialize the semaphore on the first call. Subsequent calls are no-ops.
pub fn init_global_rate_limiter(max_concurrent: usize) {
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    // Use blocking initialization - this should be called from main thread
    let _ = GLOBAL_RATE_LIMITER.set(semaphore);
    // If already set, ignore the error
}

/// Get the global rate limiter semaphore
fn get_global_rate_limiter() -> &'static Arc<Semaphore> {
    GLOBAL_RATE_LIMITER.get().expect("Rate limiter not initialized. Call init_global_rate_limiter first.")
}

/// A wrapper around any Llm implementation that adds rate limiting
/// by introducing a delay before each API call.
/// 
/// This also uses a global semaphore to limit concurrent requests across all agents.
pub struct RateLimitedLlm {
    inner: Arc<dyn Llm>,
    delay_ms: u64,
}

impl RateLimitedLlm {
    /// Create a new rate-limited LLM wrapper
    ///
    /// # Arguments
    /// * `inner` - The underlying LLM implementation
    /// * `delay_ms` - Delay in milliseconds before each API call
    pub fn new(inner: Arc<dyn Llm>, delay_ms: u64) -> Self {
        Self { inner, delay_ms }
    }

    /// Create with 2-second delay (for <30 calls per minute limit)
    pub fn with_default_delay(inner: Arc<dyn Llm>) -> Self {
        Self::new(inner, 2000) // 2 seconds = 2000ms
    }
}

#[async_trait]
impl Llm for RateLimitedLlm {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn generate_content(
        &self,
        req: LlmRequest,
        stream: bool,
    ) -> adk_core::Result<LlmResponseStream> {
        // Acquire permit from global semaphore (limits concurrent requests)
        let _permit = get_global_rate_limiter().acquire().await;
        
        // Wait before making the API call
        sleep(Duration::from_millis(self.delay_ms)).await;
        
        // Delegate to the inner LLM
        self.inner.generate_content(req, stream).await
    }
}