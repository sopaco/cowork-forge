use adk_core::{Agent, Event, AdkError, InvocationContext};
use async_trait::async_trait;
use std::sync::Arc;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use futures::{Stream, Future};
use dialoguer::{Select, Input, theme::ColorfulTheme};

pub struct ResilientAgent {
    inner: Arc<dyn Agent>,
    subs: Vec<Arc<dyn Agent>>,
}

impl ResilientAgent {
    pub fn new(inner: Arc<dyn Agent>) -> Self {
        Self {
            inner: inner.clone(),
            subs: vec![inner],
        }
    }
}

type AgentOutput = Pin<Box<dyn Stream<Item = Result<Event, AdkError>> + Send>>;

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
                // Wrap the stream to handle errors during iteration
                Ok(Box::pin(ResilientStream::new(
                    self.inner.clone(),
                    context,
                    stream
                )))
            },
            Err(e) => {
                // Handle immediate errors (same logic as before, but adapted for consistency)
                // We can't use the Stream wrapper here easily without a stream.
                // But we can just use the sync logic here since we are in async fn.
                let err_msg = e.to_string();
                if err_msg.contains("Max iterations") {
                     return self.handle_error(context, e).await;
                }
                Err(e)
            }
        }
    }
}

impl ResilientAgent {
    // Helper for immediate errors (recursion in async fn)
    async fn handle_error(&self, context: Arc<dyn InvocationContext>, e: AdkError) -> Result<AgentOutput, AdkError> {
         println!("\n⚠️  Agent '{}' encountered error: {}", self.name(), e);
         println!("The agent loop limit has been exceeded.");
         
         let selections = &["Retry (reset counter)", "Provide Guidance & Retry", "Abort"];
         let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("How would you like to proceed?")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap_or(2);

         match selection {
            0 => {
                println!("🔄 Retrying agent execution...");
                return self.run(context).await;
            },
            1 => {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Please provide guidance for the agent")
                    .interact_text()
                    .unwrap_or_default();
                
                if !input.is_empty() {
                    println!("(Note: User guidance provided: '{}' - but context injection is not implemented. Retrying anyway.)", input);
                }
                println!("🔄 Retrying with new guidance...");
                return self.run(context).await;
            },
            _ => return Err(e),
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
    agent_name: String, // Cached for logging
}

impl ResilientStream {
    fn new(
        inner_agent: Arc<dyn Agent>,
        context: Arc<dyn InvocationContext>,
        stream: AgentOutput,
    ) -> Self {
        let agent_name = inner_agent.name().to_string();
        Self {
            inner_agent,
            context,
            state: StreamState::Streaming(stream),
            agent_name,
        }
    }

    fn start_retry(&mut self) {
        let agent = self.inner_agent.clone();
        let ctx = self.context.clone();
        // Create the future for running the agent again
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
                            // Intercept error
                            let err_msg = e.to_string();
                            if err_msg.contains("Max iterations") {
                                println!("\n⚠️  Agent '{}' encountered error during stream: {}", self.agent_name, err_msg);
                                println!("The agent loop limit has been exceeded.");
                                
                                // Blocking interaction
                                let selections = &["Retry (reset counter)", "Provide Guidance & Retry", "Abort"];
                                let selection = Select::with_theme(&ColorfulTheme::default())
                                    .with_prompt("How would you like to proceed?")
                                    .default(0)
                                    .items(&selections[..])
                                    .interact()
                                    .unwrap_or(2);

                                match selection {
                                    0 => {
                                        println!("🔄 Retrying agent execution...");
                                        self.start_retry();
                                        continue; // Loop to poll the new state
                                    },
                                    1 => {
                                        let input: String = Input::with_theme(&ColorfulTheme::default())
                                            .with_prompt("Please provide guidance for the agent")
                                            .interact_text()
                                            .unwrap_or_default();
                                        if !input.is_empty() {
                                            println!("(Note: User guidance provided: '{}' - but context injection is not implemented. Retrying anyway.)", input);
                                        }
                                        println!("🔄 Retrying with new guidance...");
                                        self.start_retry();
                                        continue;
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
                            // Retry successful, got new stream
                            // Wrap it recursively? No, just replace current stream
                            // But wait, the new stream might also fail later.
                            // So we just go back to Streaming state with the new stream.
                            self.state = StreamState::Streaming(new_stream);
                            continue; // Loop to poll the new stream
                        },
                        Poll::Ready(Err(e)) => {
                            // Retry failed immediately
                            // We could offer HITL again here, but let's just error out for now to avoid infinite loops of immediate errors
                            // Or better: recurse logic?
                            // For simplicity, return the error.
                            return Poll::Ready(Some(Err(e)));
                        },
                        Poll::Pending => return Poll::Pending,
                    }
                }
            }
        }
    }
}
