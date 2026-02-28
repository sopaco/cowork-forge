// Token Bucket Rate Limiter for LLM API calls
// 
// This module implements a token bucket algorithm that allows burst requests
// while maintaining an average rate limit. This is more efficient than fixed
// delays as it allows immediate execution when tokens are available.

use adk_core::{Llm, LlmRequest, LlmResponseStream, AdkError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

/// Default configuration for rate limiting
const DEFAULT_MAX_BURST: u32 = 5;           // Allow up to 5 burst requests
const DEFAULT_RATE_LIMIT_PER_MINUTE: u32 = 30; // 30 requests per minute

/// Token bucket state
struct TokenBucketState {
    available_tokens: u32,
    last_refill: Instant,
}

/// A rate limiter using token bucket algorithm
/// 
/// This allows burst requests up to `max_burst` while maintaining an average
/// rate of `rate_limit_per_minute` requests per minute.
/// 
/// Benefits over fixed delay:
/// - Allows immediate execution when tokens are available
/// - Supports burst scenarios (e.g., initial stage execution)
/// - More efficient token usage overall
pub struct TokenBucketRateLimiter {
    inner: Arc<dyn Llm>,
    state: Mutex<TokenBucketState>,
    max_tokens: u32,
    refill_interval: Duration,
    tokens_per_refill: u32,
    max_retries: u32,
}

impl TokenBucketRateLimiter {
    /// Create a new token bucket rate limiter
    ///
    /// # Arguments
    /// * `inner` - The underlying LLM implementation
    /// * `max_burst` - Maximum number of burst requests allowed
    /// * `rate_limit_per_minute` - Average rate limit (requests per minute)
    pub fn new(
        inner: Arc<dyn Llm>,
        max_burst: u32,
        rate_limit_per_minute: u32,
    ) -> Self {
        // Calculate refill interval: we want to allow `rate_limit_per_minute` requests
        // per minute, so we add one token every `60/rate_limit_per_minute` seconds
        let refill_interval_secs = 60.0 / rate_limit_per_minute as f64;
        
        Self {
            inner,
            state: Mutex::new(TokenBucketState {
                available_tokens: max_burst,
                last_refill: Instant::now(),
            }),
            max_tokens: max_burst,
            refill_interval: Duration::from_secs_f64(refill_interval_secs),
            tokens_per_refill: 1,
            max_retries: 5,
        }
    }

    /// Create with default configuration (5 burst, 30 req/min)
    pub fn with_defaults(inner: Arc<dyn Llm>) -> Self {
        Self::new(inner, DEFAULT_MAX_BURST, DEFAULT_RATE_LIMIT_PER_MINUTE)
    }

    /// Refill tokens based on elapsed time
    fn refill_tokens(&self, state: &mut TokenBucketState) {
        let elapsed = state.last_refill.elapsed();
        let refill_periods = elapsed.as_nanos() / self.refill_interval.as_nanos();
        
        if refill_periods > 0 {
            let tokens_to_add = (refill_periods as u32) * self.tokens_per_refill;
            state.available_tokens = (state.available_tokens + tokens_to_add).min(self.max_tokens);
            state.last_refill += self.refill_interval * refill_periods as u32;
        }
    }

    /// Acquire a token, waiting if necessary
    async fn acquire_token(&self) {
        loop {
            let wait_duration = {
                let mut state = self.state.lock().await;
                self.refill_tokens(&mut state);
                
                if state.available_tokens > 0 {
                    state.available_tokens -= 1;
                    tracing::debug!(
                        "[TokenBucket] Token acquired, {} remaining",
                        state.available_tokens
                    );
                    return;
                }
                
                // Calculate how long until next token
                self.refill_interval
            };
            
            tracing::debug!(
                "[TokenBucket] No tokens available, waiting {:?}",
                wait_duration
            );
            sleep(wait_duration).await;
        }
    }

    /// Check if an error message indicates a rate limit error (429)
    fn is_rate_limit_error(error: &AdkError) -> bool {
        let error_str = format!("{:?}", error).to_lowercase();
        error_str.contains("429") || 
        error_str.contains("too many requests") || 
        error_str.contains("rate limit") ||
        error_str.contains("quota")
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
impl Llm for TokenBucketRateLimiter {
    fn name(&self) -> &str {
        self.inner.name()
    }

    async fn generate_content(
        &self,
        req: LlmRequest,
        stream: bool,
    ) -> Result<LlmResponseStream, AdkError> {
        // Try the request with exponential backoff on 429 errors
        let mut last_error: Option<AdkError> = None;
        
        for attempt in 0..self.max_retries {
            // Acquire token before each attempt (including retries)
            // This ensures we respect rate limits even after 429 backoff
            self.acquire_token().await;
            
            // Clone the request for retry (since it might be consumed)
            let req_clone = req.clone();
            
            match self.inner.generate_content(req_clone, stream).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Check if this is a rate limit error
                    if Self::is_rate_limit_error(&e) {
                        last_error = Some(e);
                        
                        // Calculate backoff delay
                        let backoff = Self::calculate_backoff(attempt);
                        eprintln!(
                            "[TokenBucket] Rate limit hit (attempt {}/{}), waiting {:?} before retry...",
                            attempt + 1, self.max_retries, backoff
                        );
                        
                        // Wait with exponential backoff
                        sleep(backoff).await;
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

// ============================================================================
// Backward compatibility: Keep the old RateLimitedLlm name as an alias
// ============================================================================

/// Type alias for backward compatibility
/// The new implementation uses token bucket algorithm
pub type RateLimitedLlm = TokenBucketRateLimiter;

/// Legacy function for backward compatibility
/// Creates a rate limiter with default settings
pub fn init_global_rate_limiter(_max_concurrent: usize) {
    // No-op: The new token bucket implementation doesn't need global semaphore
    // This function is kept for backward compatibility
    tracing::info!("[TokenBucket] Rate limiter initialized (no global semaphore needed)");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_creation() {
        let state = TokenBucketState {
            available_tokens: 5,
            last_refill: Instant::now(),
        };
        assert_eq!(state.available_tokens, 5);
    }

    #[test]
    fn test_backoff_calculation() {
        assert_eq!(TokenBucketRateLimiter::calculate_backoff(0), Duration::from_secs(1));
        assert_eq!(TokenBucketRateLimiter::calculate_backoff(1), Duration::from_secs(4));
        assert_eq!(TokenBucketRateLimiter::calculate_backoff(2), Duration::from_secs(16));
        assert_eq!(TokenBucketRateLimiter::calculate_backoff(3), Duration::from_secs(60)); // capped
    }
}
