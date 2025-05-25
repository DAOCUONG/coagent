#[derive(Clone, Debug)]
pub struct RunConfig {
    max_llm_calls: i32,
}

impl RunConfig {
    pub fn builder() -> RunConfigBuilder {
        RunConfigBuilder {
            max_llm_calls: 500,
        }
    }

    pub fn max_llm_calls(&self) -> i32 {
        self.max_llm_calls
    }
}

#[derive(Clone, Debug)]
pub struct RunConfigBuilder {
    max_llm_calls: i32,
}

impl RunConfigBuilder {
    pub fn set_max_llm_calls(mut self, max_llm_calls: i32) -> Self {
        self.max_llm_calls = max_llm_calls;
        self
    }

    pub fn build(self) -> RunConfig {
        RunConfig {
            max_llm_calls: self.max_llm_calls,
        }
    }
}