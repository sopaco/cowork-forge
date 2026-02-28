// Rate-limited LLM wrapper with global rate limiting
use adk_core::{Llm, LlmRequest, LlmResponseStream, AdkError};
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

/// Check if an error message indicates a rate limit error (429)
fn is_rate_limit_error(error: &AdkError) -> bool {
    let error_str = format!("{:?}", error).to_lowercase();
    error_str.contains("429") || 
    error_str.contains("too many requests") || 
    error_str.contains("rate limit") ||
    error_str.contains("quota")
}

/// A wrapper around any Llm implementation that adds rate limiting
/// by introducing a delay before each API call.
/// 
/// This also uses a global semaphore to limit concurrent requests across all agents.
/// Includes exponential backoff retry for 429 rate limit errors.
pub struct RateLimitedLlm {
    inner: Arc<dyn Llm>,
    delay_ms: u64,
    max_retries: u32,
}

impl RateLimitedLlm {
    /// Create a new rate-limited LLM wrapper
    ///
    /// # Arguments
    /// * `inner` - The underlying LLM implementation
    /// * `delay_ms` - Delay in milliseconds before each API call
    pub fn new(inner: Arc<dyn Llm>, delay_ms: u64) -> Self {
        Self { inner, delay_ms, max_retries: 5 }
    }

    /// Create with 3-second delay (for ~20 calls per minute limit)
    pub fn with_default_delay(inner: Arc<dyn Llm>) -> Self {
        Self::new(inner, 3000) // 3 seconds = 3000ms
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff(attempt: u32) -> Duration {
        // Base delay: 4 seconds, max: 60 seconds
        // 4, 8, 16, 32, 60, 60, ...
        let base_seconds = 4u64;
        let max_seconds = 60u64;
        let delay = base_seconds.saturating_pow(attempt).min(max_seconds);
        Duration::from_secs(delay)
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
    ) -> Result<LlmResponseStream, AdkError> {
        // Acquire permit from global semaphore (limits concurrent requests)
        let _permit = get_global_rate_limiter().acquire().await;
        
        // Wait before making the API call
        sleep(Duration::from_millis(self.delay_ms)).await;
        
        // Try the request with exponential backoff on 429 errors
        let mut last_error: Option<AdkError> = None;
        
        for attempt in 0..self.max_retries {
            // Clone the request for retry (since it might be consumed)
            let req_clone = req.clone();
            
            match self.inner.generate_content(req_clone, stream).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Check if this is a rate limit error
                    if is_rate_limit_error(&e) {
                        last_error = Some(e);
                        
                        // Calculate backoff delay
                        let backoff = Self::calculate_backoff(attempt);
                        eprintln!(
                            "[RateLimiter] Rate limit hit (attempt {}/{}), waiting {:?} before retry...",
                            attempt + 1, self.max_retries, backoff
                        );
                        
                        // Wait with exponential backoff
                        sleep(backoff).await;
                        
                        // Re-acquire permit before retry
                        let _permit = get_global_rate_limiter().acquire().await;
                        continue;
                    } else {
                        // Non-rate-limit error, return immediately
                        return Err(e);
                    }
                }
            }
        }
        
        // All retries exhausted
        Err(last_error.expect("At least one error should exist after retry loop"))
    }
}