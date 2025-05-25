use crate::base_agent::{BaseAgent, BaseAgentMessage, BaseAgentState, BaseAgentArguments, BeforeAgentCallback, AfterAgentCallback};
use crate::common::{AgentError, Event};
use crate::invocation_context::InvocationContext;
use ractor::{Actor, ActorRef, ActorCell, ActorProcessingErr};
use tokio::sync::mpsc;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct LoopAgent {
    base: BaseAgent,
    max_iterations: Option<i32>,
}

#[async_trait]
impl Actor for LoopAgent {
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

impl LoopAgent {
    pub fn builder() -> LoopAgentBuilder {
        LoopAgentBuilder {
            name: None,
            description: None,
            sub_agents: Vec::new(),
            max_iterations: None,
            before_agent_callback: None,
            after_agent_callback: None,
        }
    }

    async fn run_async_impl(&self, context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        let sub_agents = self.base.sub_agents();
        if sub_agents.is_empty() {
            return Ok(Vec::new());
        }

        let max_iterations = self.max_iterations.unwrap_or(i32::MAX);
        let mut events = Vec::new();
        let mut iteration = 0;

        while iteration < max_iterations {
            for sub_agent in sub_agents {
                let sub_events = self.base.run_async(context.clone()).await?;
                for event in &sub_events {
                    if event.actions().escalate().unwrap_or(false) {
                        return Ok(events);
                    }
                }
                events.extend(sub_events);
            }
            iteration += 1;
        }

        Ok(events)
    }

    async fn run_live_impl(&self, _context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        Err(AgentError::UnsupportedOperation("run_live not implemented for LoopAgent".to_string()))
    }
}

#[derive(Clone, Debug)]
pub struct LoopAgentBuilder {
    name: Option<String>,
    description: Option<String>,
    sub_agents: Vec<Arc<ActorCell>>,
    max_iterations: Option<i32>,
    before_agent_callback: Option<Vec<BeforeAgentCallback>>,
    after_agent_callback: Option<Vec<AfterAgentCallback>>,
}

impl LoopAgentBuilder {
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

    pub fn max_iterations(mut self, max_iterations: i32) -> Self {
        self.max_iterations = Some(max_iterations);
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

    pub fn build(self) -> LoopAgent {
        let name = self.name.unwrap_or_default();
        let description = self.description.unwrap_or_default();
        LoopAgent {
            base: BaseAgent::new(
                name,
                description,
                self.sub_agents,
                self.before_agent_callback,
                self.after_agent_callback,
            ),
            max_iterations: self.max_iterations,
        }
    }
}