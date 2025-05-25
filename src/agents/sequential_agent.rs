use crate::base_agent::{BaseAgent, BaseAgentMessage, BaseAgentState, BaseAgentArguments, BeforeAgentCallback, AfterAgentCallback};
use crate::common::{AgentError, Event};
use crate::invocation_context::InvocationContext;
use ractor::{Actor, ActorRef, ActorCell, ActorProcessingErr};
use tokio::sync::mpsc;
use std::sync::Arc;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct SequentialAgent {
    base: BaseAgent,
}

#[async_trait]
impl Actor for SequentialAgent {
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

impl SequentialAgent {
    pub fn builder() -> SequentialAgentBuilder {
        SequentialAgentBuilder {
            name: None,
            description: None,
            sub_agents: Vec::new(),
            before_agent_callback: None,
            after_agent_callback: None,
        }
    }

    async fn run_async_impl(&self, context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        let mut events = Vec::new();
        for sub_agent in self.base.sub_agents() {
            let sub_events = self.base.run_async(context.clone()).await?;
            events.extend(sub_events);
        }
        Ok(events)
    }

    async fn run_live_impl(&self, context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        let mut events = Vec::new();
        for sub_agent in self.base.sub_agents() {
            let sub_events = self.base.run_async(context.clone()).await?;
            events.extend(sub_events);
        }
        Ok(events)
    }
}

#[derive(Clone, Debug)]
pub struct SequentialAgentBuilder {
    name: Option<String>,
    description: Option<String>,
    sub_agents: Vec<Arc<ActorCell>>,
    before_agent_callback: Option<Vec<BeforeAgentCallback>>,
    after_agent_callback: Option<Vec<AfterAgentCallback>>,
}

impl SequentialAgentBuilder {
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

    pub fn build(self) -> SequentialAgent {
        let name = self.name.unwrap_or_default();
        let description = self.description.unwrap_or_default();
        SequentialAgent {
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