use adk_core::{Agent, Event, AdkError, InvocationContext};
use crate::interaction::{InteractiveBackend, InputOption, InputResponse, MessageLevel};
use crate::event_bus::EventBus;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use futures::{Stream, Future};

type AgentOutput = Pin<Box<dyn Stream<Item = Result<Event, AdkError>> + Send>>;

pub struct ResilientAgent {
    inner: Arc<dyn Agent>,
    subs: Vec<Arc<dyn Agent>>,
    retry_count: Arc<AtomicU32>,
    interaction: Arc<dyn InteractiveBackend>,
}

impl ResilientAgent {
    const MAX_RETRY_ATTEMPTS: u32 = 3;
    
    pub fn new(inner: Arc<dyn Agent>, interaction: Arc<dyn InteractiveBackend>) -> Self {
        Self {
            inner: inner.clone(),
            subs: vec![inner],
            retry_count: Arc::new(AtomicU32::new(0)),
            interaction,
        }
    }
    
    // Helper for immediate errors (recursion in async fn)
    async fn handle_error(&self, context: Arc<dyn InvocationContext>, e: AdkError) -> Result<AgentOutput, AdkError> {
        let current_retry = self.retry_count.fetch_add(1, Ordering::SeqCst);
        
        // Check if max retry attempts reached
        if current_retry >= Self::MAX_RETRY_ATTEMPTS {
            self.interaction.show_message(
                MessageLevel::Error,
                format!("Maximum retry attempts ({}) reached for agent '{}'", Self::MAX_RETRY_ATTEMPTS, self.name())
            ).await;
            self.retry_count.store(0, Ordering::SeqCst);
            return Err(AdkError::Tool(format!(
                "Agent '{}' failed after {} retry attempts", 
                self.name(), 
                Self::MAX_RETRY_ATTEMPTS
            )));
        }
        
        self.interaction.show_message(
            MessageLevel::Warning,
            format!("Agent '{}' encountered error: {}", self.name(), e)
        ).await;
        
        let options = vec![
            InputOption {
                id: "retry".to_string(),
                label: "Retry (reset counter)".to_string(),
                description: Some("Retry the agent execution".to_string()),
            },
            InputOption {
                id: "guidance".to_string(),
                label: "Provide Guidance & Retry".to_string(),
                description: Some("Provide feedback to guide the agent".to_string()),
            },
            InputOption {
                id: "abort".to_string(),
                label: "Abort".to_string(),
                description: Some("Stop the execution".to_string()),
            },
        ];
        
        let response = self.interaction.request_input(
            "How would you like to proceed?",
            options,
            None
        ).await.map_err(|e| AdkError::Tool(format!("Input error: {}", e)))?;

        match response {
            InputResponse::Selection(id) => match id.as_str() {
                "retry" => {
                    self.interaction.show_message(MessageLevel::Info, "🔄 Retrying agent execution...".to_string()).await;
                    return self.run(context).await;
                },
                "guidance" => {
                    // In the future, we could inject guidance into context
                    self.interaction.show_message(MessageLevel::Info, "🔄 Retrying with new guidance...".to_string()).await;
                    return self.run(context).await;
                },
                "abort" => {
                    self.retry_count.store(0, Ordering::SeqCst);
                    return Err(e);
                },
                _ => Err(e),
            },
            InputResponse::Cancel => {
                self.retry_count.store(0, Ordering::SeqCst);
                Err(e)
            },
            _ => Err(e),
        }
    }
}

#[async_trait]
impl Agent for ResilientAgent {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn sub_agents(&self) -> &[Arc<dyn Agent>] {
        &self.subs
    }

    async fn run(&self, context: Arc<dyn InvocationContext>) -> Result<AgentOutput, AdkError> {
        // Initial run
        match self.inner.run(context.clone()).await {
            Ok(stream) => {
                // Success - reset retry counter
                self.retry_count.store(0, Ordering::SeqCst);
                // Wrap the stream to handle errors during iteration
                Ok(Box::pin(ResilientStream::new(
                    self.inner.clone(),
                    context,
                    stream,
                    self.retry_count.clone(),
                    self.interaction.clone(),
                )))
            },
            Err(e) => {
                // Handle immediate errors
                let err_msg = e.to_string();
                if err_msg.contains("Max iterations") {
                     return self.handle_error(context, e).await;
                }
                Err(e)
            }
        }
    }
}

// ============================================================================
// ResilientStream Implementation
// ============================================================================

enum StreamState {
    Streaming(AgentOutput),
    Retrying(Pin<Box<dyn Future<Output = Result<AgentOutput, AdkError>> + Send>>),
}

struct ResilientStream {
    inner_agent: Arc<dyn Agent>,
    context: Arc<dyn InvocationContext>,
    state: StreamState,
    agent_name: String,
    retry_count: Arc<AtomicU32>,
    interaction: Arc<dyn InteractiveBackend>,
}

impl ResilientStream {
    fn new(
        inner_agent: Arc<dyn Agent>,
        context: Arc<dyn InvocationContext>,
        stream: AgentOutput,
        retry_count: Arc<AtomicU32>,
        interaction: Arc<dyn InteractiveBackend>,
    ) -> Self {
        let agent_name = inner_agent.name().to_string();
        Self {
            inner_agent,
            context,
            state: StreamState::Streaming(stream),
            agent_name,
            retry_count,
            interaction,
        }
    }

    fn start_retry(&mut self) {
        let agent = self.inner_agent.clone();
        let ctx = self.context.clone();
        let fut = Box::pin(async move {
            agent.run(ctx).await
        });
        self.state = StreamState::Retrying(fut);
    }
}

impl Stream for ResilientStream {
    type Item = Result<Event, AdkError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match &mut self.state {
                StreamState::Streaming(stream) => {
                    match stream.as_mut().poll_next(cx) {
                        Poll::Ready(Some(Err(e))) => {
                            let err_msg = e.to_string();
                            if err_msg.contains("Max iterations") {
                                // Async interaction is not possible during stream polling
                                // We'll need to show message and offer retry options
                                // For now, use blocking approach (CLI backend handles this synchronously)
                                
                                let options = vec![
                                    InputOption {
                                        id: "retry".to_string(),
                                        label: "Retry (reset counter)".to_string(),
                                        description: Some("Retry the agent execution".to_string()),
                                    },
                                    InputOption {
                                        id: "guidance".to_string(),
                                        label: "Provide Guidance & Retry".to_string(),
                                        description: Some("Provide feedback to guide the agent".to_string()),
                                    },
                                    InputOption {
                                        id: "abort".to_string(),
                                        label: "Abort".to_string(),
                                        description: Some("Stop the execution".to_string()),
                                    },
                                ];

                                // This will block for CLI, but for GUI it needs to be handled differently
                                // For now, we'll use tokio::task::block_in_place to allow async during blocking
                                let interaction_clone = self.interaction.clone();
                                let response = tokio::task::block_in_place(|| {
                                    tokio::runtime::Handle::current().block_on(async {
                                        interaction_clone.request_input(
                                            "How would you like to proceed?",
                                            options,
                                            None
                                        ).await
                                    })
                                }).map_err(|e| AdkError::Tool(format!("Input error: {}", e)))?;

                                match response {
                                    InputResponse::Selection(id) => match id.as_str() {
                                        "retry" | "guidance" => {
                                            self.start_retry();
                                            continue;
                                        },
                                        "abort" => return Poll::Ready(Some(Err(e))),
                                        _ => return Poll::Ready(Some(Err(e))),
                                    },
                                    _ => return Poll::Ready(Some(Err(e))),
                                }
                            }
                            return Poll::Ready(Some(Err(e)));
                        },
                        Poll::Ready(other) => return Poll::Ready(other),
                        Poll::Pending => return Poll::Pending,
                    }
                },
                StreamState::Retrying(fut) => {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(Ok(new_stream)) => {
                            self.state = StreamState::Streaming(new_stream);
                            continue;
                        },
                        Poll::Ready(Err(e)) => {
                            return Poll::Ready(Some(Err(e)));
                        },
                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}
