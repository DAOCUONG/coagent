use crate::common::{AgentError, BaseArtifactService, BaseSessionService, Content, LiveRequestQueue, RunConfig, Session};
use ractor::ActorCell;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct InvocationContext {
    session_service: BaseSessionService,
    artifact_service: BaseArtifactService,
    live_request_queue: Option<LiveRequestQueue>,
    branch: Option<String>,
    invocation_id: String,
    agent: Arc<ActorCell>,
    session: Session,
    user_content: Option<Content>,
    run_config: RunConfig,
    end_invocation: bool,
    invocation_cost_manager: InvocationCostManager,
}

impl InvocationContext {
    pub fn create(
        session_service: BaseSessionService,
        artifact_service: BaseArtifactService,
        invocation_id: String,
        agent: Arc<ActorCell>,
        session: Session,
        user_content: Option<Content>,
        run_config: RunConfig,
    ) -> Self {
        InvocationContext {
            session_service,
            artifact_service,
            live_request_queue: None,
            branch: None,
            invocation_id,
            agent,
            session,
            user_content,
            run_config,
            end_invocation: false,
            invocation_cost_manager: InvocationCostManager { number_of_llm_calls: 0 },
        }
    }

    pub fn copy_of(other: &Self) -> Self {
        InvocationContext {
            session_service: other.session_service.clone(),
            artifact_service: other.artifact_service.clone(),
            live_request_queue: other.live_request_queue.clone(),
            branch: other.branch.clone(),
            invocation_id: other.invocation_id.clone(),
            agent: other.agent.clone(),
            session: other.session.clone(),
            user_content: other.user_content.clone(),
            run_config: other.run_config.clone(),
            end_invocation: other.end_invocation,
            invocation_cost_manager: other.invocation_cost_manager.clone(),
        }
    }

    pub fn session_service(&self) -> &BaseSessionService {
        &self.session_service
    }

    pub fn artifact_service(&self) -> &BaseArtifactService {
        &self.artifact_service
    }

    pub fn live_request_queue(&self) -> Option<&LiveRequestQueue> {
        self.live_request_queue.as_ref()
    }

    pub fn invocation_id(&self) -> &str {
        &self.invocation_id
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn set_branch(&mut self, branch: Option<String>) {
        self.branch = branch;
    }

    pub fn agent(&self) -> Arc<ActorCell> {
        self.agent.clone()
    }

    pub fn set_agent(&mut self, agent: Arc<ActorCell>) {
        self.agent = agent;
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn user_content(&self) -> Option<&Content> {
        self.user_content.as_ref()
    }

    pub fn run_config(&self) -> &RunConfig {
        &self.run_config
    }

    pub fn end_invocation(&self) -> bool {
        self.end_invocation
    }

    pub fn app_name(&self) -> &str {
        self.session.app_name()
    }

    pub fn user_id(&self) -> &str {
        self.session.user_id()
    }

    pub fn new_invocation_context_id() -> String {
        format!("e-{}", Uuid::new_v4())
    }

    pub fn increment_llm_calls_count(&mut self) -> Result<(), AgentError> {
        self.invocation_cost_manager.increment_and_enforce_llm_calls_limit(&self.run_config)
    }
}

#[derive(Clone, Debug)]
struct InvocationCostManager {
    number_of_llm_calls: i32,
}

impl InvocationCostManager {
    fn increment_and_enforce_llm_calls_limit(&mut self, run_config: &RunConfig) -> Result<(), AgentError> {
        self.number_of_llm_calls += 1;
        if run_config.max_llm_calls() > 0 && self.number_of_llm_calls > run_config.max_llm_calls() {
            return Err(AgentError::LlmCallsLimitExceeded(
                format!("Max number of LLM calls limit of {} exceeded", run_config.max_llm_calls())
            ));
        }
        Ok(())
    }
}