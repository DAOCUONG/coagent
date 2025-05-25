use crate::common::{AgentError, Content, Event, EventActions};
use crate::invocation_context::InvocationContext;
use crate::callback_context::CallbackContext;
use ractor::{Actor, ActorRef, ActorCell, ActorProcessingErr};
use tokio::sync::mpsc;
use std::sync::Arc;
use async_trait::async_trait;

pub type BeforeAgentCallback = Arc<dyn Fn(CallbackContext) -> tokio::sync::oneshot::Receiver<Option<Content>> + Send + Sync>;
pub type AfterAgentCallback = Arc<dyn Fn(CallbackContext) -> tokio::sync::oneshot::Receiver<Option<Content>> + Send + Sync>;

#[derive(Clone, Debug)]
pub struct BaseAgent {
    pub name: String,
    pub description: String,
    pub parent_agent: Option<Arc<ActorCell>>,
    pub sub_agents: Vec<Arc<ActorCell>>,
    pub before_agent_callback: Option<Vec<BeforeAgentCallback>>,
    pub after_agent_callback: Option<Vec<AfterAgentCallback>>,
}

#[async_trait]
impl Actor for BaseAgent {
    type Msg = BaseAgentMessage;
    type State = BaseAgentState;
    type Arguments = BaseAgentArguments;

    async fn pre_start(&self, _this_actor: ActorRef<Self>, _args: Self::Arguments) -> Result<Self::State, ActorProcessingErr> {
        Ok(BaseAgentState {
            name: self.name.clone(),
            description: self.description.clone(),
            parent_agent: self.parent_agent.clone(),
            sub_agents: self.sub_agents.clone(),
            before_agent_callback: self.before_agent_callback.clone(),
            after_agent_callback: self.after_agent_callback.clone(),
        })
    }

    async fn handle(
        &self,
        _this_actor: ActorRef<Self>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            BaseAgentMessage::RunAsync { context, sender } => {
                let events = self.run_async_impl(&context).await?;
                sender.send(events).map_err(|_| ActorProcessingErr::new("Failed to send events"))?;
            }
            BaseAgentMessage::RunLive { context, sender } => {
                let events = self.run_live_impl(&context).await?;
                sender.send(events).map_err(|_| ActorProcessingErr::new("Failed to send events"))?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum BaseAgentMessage {
    RunAsync {
        context: InvocationContext,
        sender: mpsc::Sender<Vec<Event>>,
    },
    RunLive {
        context: InvocationContext,
        sender: mpsc::Sender<Vec<Event>>,
    },
}

#[derive(Clone, Debug)]
pub struct BaseAgentState {
    pub name: String,
    pub description: String,
    pub parent_agent: Option<Arc<ActorCell>>,
    pub sub_agents: Vec<Arc<ActorCell>>,
    pub before_agent_callback: Option<Vec<BeforeAgentCallback>>,
    pub after_agent_callback: Option<Vec<AfterAgentCallback>>,
}

#[derive(Clone, Debug)]
pub struct BaseAgentArguments {
    pub name: String,
    pub description: String,
    pub sub_agents: Vec<Arc<ActorCell>>,
    pub before_agent_callback: Option<Vec<BeforeAgentCallback>>,
    pub after_agent_callback: Option<Vec<AfterAgentCallback>>,
}

impl BaseAgent {
    pub fn new(
        name: String,
        description: String,
        sub_agents: Vec<Arc<ActorCell>>,
        before_agent_callback: Option<Vec<BeforeAgentCallback>>,
        after_agent_callback: Option<Vec<AfterAgentCallback>>,
    ) -> Self {
        let parent_agent = None;
        BaseAgent {
            name,
            description,
            parent_agent,
            sub_agents,
            before_agent_callback,
            after_agent_callback,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn parent_agent(&self) -> Option<Arc<ActorCell>> {
        self.parent_agent.clone()
    }

    pub fn set_parent_agent(&mut self, parent: Arc<ActorCell>) {
        self.parent_agent = Some(parent);
    }

    pub fn root_agent(&self) -> Arc<ActorCell> {
        let mut current = Arc::new(ActorCell::new_null());
        if let Some(parent) = &self.parent_agent {
            current = parent.clone();
        }
        current
    }

    pub fn find_agent(&self, name: &str) -> Option<Arc<ActorCell>> {
        if self.name == name {
            return Some(Arc::new(ActorCell::new_null()));
        }
        self.find_sub_agent(name)
    }

    pub fn find_sub_agent(&self, name: &str) -> Option<Arc<ActorCell>> {
        for sub_agent in &self.sub_agents {
            // Placeholder: Implement actor query
        }
        None
    }

    pub fn sub_agents(&self) -> &Vec<Arc<ActorCell>> {
        &self.sub_agents
    }

    pub fn before_agent_callback(&self) -> Option<&Vec<BeforeAgentCallback>> {
        self.before_agent_callback.as_ref()
    }

    pub fn after_agent_callback(&self) -> Option<&Vec<AfterAgentCallback>> {
        self.after_agent_callback.as_ref()
    }

    pub fn create_invocation_context(&self, parent_context: &InvocationContext) -> InvocationContext {
        let mut context = InvocationContext::copy_of(parent_context);
        context.set_agent(Arc::new(ActorCell::new_null()));
        if let Some(branch) = parent_context.branch().filter(|s| !s.is_empty()) {
            context.set_branch(Some(format!("{}.{}", branch, self.name)));
        } else {
            context.set_branch(Some(self.name.clone()));
        }
        context
    }

    pub async fn run_async(&self, parent_context: InvocationContext) -> Result<Vec<Event>, AgentError> {
        let mut context = self.create_invocation_context(&parent_context);
        let mut events = Vec::new();

        if let Some(callbacks) = &self.before_agent_callback {
            for callback in callbacks {
                let callback_context = CallbackContext::new(context.clone(), None);
                let receiver = callback(callback_context);
                if let Ok(Some(content)) = receiver.await {
                    events.push(Event {
                        id: Event::generate_event_id(),
                        invocation_id: context.invocation_id().to_string(),
                        author: self.name.clone(),
                        branch: context.branch().map(|s| s.to_string()),
                        actions: EventActions::builder().build(),
                        content: Some(content),
                        final_response: false,
                    });
                    if context.end_invocation() {
                        return Ok(events);
                    }
                }
            }
        }

        let main_events = self.run_async_impl(&context).await?;
        events.extend(main_events);

        if let Some(callbacks) = &self.after_agent_callback {
            for callback in callbacks {
                let callback_context = CallbackContext::new(context.clone(), None);
                let receiver = callback(callback_context);
                if let Ok(Some(content)) = receiver.await {
                    events.push(Event {
                        id: Event::generate_event_id(),
                        invocation_id: context.invocation_id().to_string(),
                        author: self.name.clone(),
                        branch: context.branch().map(|s| s.to_string()),
                        actions: EventActions::builder().build(),
                        content: Some(content),
                        final_response: false,
                    });
                }
            }
        }

        Ok(events)
    }

    async fn run_async_impl(&self, _context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        Ok(Vec::new())
    }

    async fn run_live_impl(&self, _context: &InvocationContext) -> Result<Vec<Event>, AgentError> {
        Err(AgentError::UnsupportedOperation("run_live not implemented".to_string()))
    }
}