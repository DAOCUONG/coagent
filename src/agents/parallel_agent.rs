use crate::base_agent::{BaseAgent, BaseAgentMessage, BaseAgentState, BaseAgentArguments, BeforeAgentCallback, AfterAgentCallback};
use crate::common::{AgentError, Event};
use crate::invocation_context::InvocationContext;
use ractor::{Actor, ActorRef, ActorCell, ActorProcessingErr};
use tokio::sync::mpsc;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct ParallelAgent {
    base: BaseAgent,
}

#[async_trait]
impl Actor for ParallelAgent {
    type Msg = BaseAgentMessage;
    type State = BaseAgentState;
    type Arguments = BaseAgentArguments;

    async fn pre_start(&self, this_actor: ActorRef<Self>, args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        self.base.pre_start(this_actor, args).await
    }

    async fn handle(
        &self,
        this_actor: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        self.base.handle(this_actor, message, state).await
    }
}

impl ParallelAgent {
    pub fn builder() -> ParallelAgentBuilder {
        ParallelAgentBuilder {
            name: None,
            description: None,
            sub_agents: Vec::new(),
            before_agent_callback: None,
            after_agent_callback: None,
        }
    }

    async fn run_async_impl(&self, context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        let mut handles = Vec::new();
        for sub_agent in self.base.sub_agents() {
            let context = context.clone();
            handles.push(tokio::spawn(async move {
                // Placeholder: Send RunAsync message to sub-agent
                let (tx, rx) = mpsc::channel(1);
                rx.recv().await.unwrap_or_default()
            }));
        }
        let mut events = Vec::new();
        for handle in handles {
            events.extend(handle.await.unwrap());
        }
        Ok(events)
    }

    async fn run_live_impl(&self, _context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        Err(AgentError::UnsupportedOperation("run_live not implemented for ParallelAgent".to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct ParallelAgentBuilder {
    name: Option<String>,
    description: Option<String>,
    sub_agents: Vec<Arc<ActorCell>>,
    before_agent_callback: Option<Vec<BeforeAgentCallback>>,
    after_agent_callback: Option<Vec<AfterAgentCallback>>,
}

impl ParallelAgentBuilder {
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn sub_agents(mut self, sub_agents: Vec<Arc<ActorCell>>) -> Self {
        self.sub_agents = sub_agents;
        self
    }

    pub fn before_agent_callback(mut self, callback: BeforeAgentCallback) -> Self {
        self.before_agent_callback = Some(vec![callback]);
        self
    }

    pub fn after_agent_callback(mut self, callback: AfterAgentCallback) -> Self {
        self.after_agent_callback = Some(vec![callback]);
        self
    }

    pub fn build(self) -> ParallelAgent {
        let name = self.name.unwrap_or_default();
        let description = self.description.unwrap_or_default();
        ParallelAgent {
            base: BaseAgent::new(
                name,
                description,
                self.sub_agents,
                self.before_agent_callback,
                self.after_agent_callback,
            ),
        }
    }
}