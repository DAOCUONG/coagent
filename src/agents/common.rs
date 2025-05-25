use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Content;

#[derive(Clone, Debug)]
pub struct Event {
    pub id: String,
    pub invocation_id: String,
    pub author: String,
    pub branch: Option<String>,
    pub actions: EventActions,
    pub content: Option<Content>,
    pub final_response: bool,
}

impl Event {
    pub fn builder() -> EventBuilder {
        EventBuilder {
            id: Event::generate_event_id(),
            invocation_id: String::new(),
            author: String::new(),
            branch: None,
            actions: EventActions::builder().build(),
            content: None,
            final_response: false,
        }
    }

    pub fn generate_event_id() -> String {
        format!("e-{}", Uuid::new_v4())
    }

    pub fn actions(&self) -> &EventActions {
        &self.actions
    }

    pub fn final_response(&self) -> bool {
        self.final_response
    }

    pub fn content(&self) -> Option<&Content> {
        self.content.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct EventBuilder {
    id: String,
    invocation_id: String,
    author: String,
    branch: Option<String>,
    actions: EventActions,
    content: Option<Content>,
    final_response: bool,
}

impl EventBuilder {
    pub fn id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    pub fn invocation_id(mut self, invocation_id: String) -> Self {
        self.invocation_id = invocation_id;
        self
    }

    pub fn author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    pub fn branch(mut self, branch: Option<String>) -> Self {
        self.branch = branch;
        self
    }

    pub fn actions(mut self, actions: EventActions) -> Self {
        self.actions = actions;
        self
    }

    pub fn content(mut self, content: Option<Content>) -> Self {
        self.content = content;
        self
    }

    pub fn build(self) -> Event {
        Event {
            id: self.id,
            invocation_id: self.invocation_id,
            author: self.author,
            branch: self.branch,
            actions: self.actions,
            content: self.content,
            final_response: self.final_response,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventActions {
    state_delta: HashMap<String, serde_json::Value>,
    escalate: bool,
    artifact_delta: HashMap<String, Part>,
}

impl EventActions {
    pub fn builder() -> EventActionsBuilder {
        EventActionsBuilder {
            state_delta: HashMap::new(),
            escalate: false,
            artifact_delta: HashMap::new(),
        }
    }

    pub fn state_delta(&mut self) -> &mut HashMap<String, serde_json::Value> {
        &mut self.state_delta
    }

    pub fn escalate(&self) -> Option<bool> {
        Some(self.escalate)
    }

    pub fn artifact_delta(&mut self) -> &mut HashMap<String, Part> {
        &mut self.artifact_delta
    }
}

#[derive(Clone, Debug)]
pub struct EventActionsBuilder {
    state_delta: HashMap<String, serde_json::Value>,
    escalate: bool,
    artifact_delta: HashMap<String, Part>,
}

impl EventActionsBuilder {
    pub fn build(self) -> EventActions {
        EventActions {
            state_delta: self.state_delta,
            escalate: self.escalate,
            artifact_delta: self.artifact_delta,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Part;

#[derive(Clone, Debug)]
pub struct Session {
    state: HashMap<String, serde_json::Value>,
    app_name: String,
    user_id: String,
    id: String,
}

impl Session {
    pub fn state(&self) -> &HashMap<String, serde_json::Value> {
        &self.state
    }

    pub fn app_name(&self) -> &str {
        &self.app_name
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub struct BaseSessionService;

#[derive(Clone, Debug)]
pub struct BaseArtifactService;

#[derive(Clone, Debug)]
pub struct LiveRequestQueue;

#[derive(Clone, Debug)]
pub struct BaseTool;

#[derive(Debug)]
pub enum AgentError {
    LlmCallsLimitExceeded(String),
    UnsupportedOperation(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AgentError::LlmCallsLimitExceeded(msg) => write!(f, "{}", msg),
            AgentError::UnsupportedOperation(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AgentError {}