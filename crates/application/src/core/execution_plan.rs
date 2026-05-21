use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, MessageRequest, MessageRole};
use agente_infrastructure::adapters::util::load_file_installed::load_file_installed;

#[derive(Debug)]
pub struct ExecutionPlan {
    pub is_complex: bool,
    pub size: usize,
    pub steps: Vec<Step>,
}

impl ExecutionPlan {
    pub async fn generate(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<Self, Error> {
        let default_steps = vec![Step {
            prompt: input.clone(),
            result: None,
        }];

        let (steps, is_complex) = if input.len() < 20 {
            (default_steps, false)
        } else if Self::is_prompt_complex(agent, input.clone()).await? {
            // @TODO: should we reasoning this prompt??
            let results = Self::split_prompt(agent, input).await?;
            let steps = results.iter().map(|r| Step::new(r.to_string())).collect();
            (steps, true)
        } else {
            (default_steps, false)
        };

        let plan = Self {
            is_complex,
            size: steps.len(),
            steps,
        };
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("PLAN: {:#?}", plan);
        }

        Ok(plan)
    }

    pub async fn is_done(
        &self,
        agent: &Box<dyn AiProvider>,
    ) -> Result<(bool, String), Error> {
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("FINISHED PLAN: {:#?}", self.steps);
        }

        let results = self
            .steps
            .iter()
            .map(|step| {
                format!(
                    "Message: {}, Result: {}",
                    step.prompt,
                    step.result.clone().unwrap_or("Not finished".to_string())
                )
            })
            .collect::<Vec<_>>();

        // @TODO: need to consider last execution plan execution summary
        let execution_summary_system_prompt =
            "Based on the the next messages, determine if the goal as reached and if dont, describe what is needed to finish it, but never add anything that we didn't asked in the inital goal unless its really needed to finish it"
                .to_string();
        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: execution_summary_system_prompt,
            },
            MessageRequest {
                role: MessageRole::User,
                content: results.join(";"),
            },
        ];

        let execution_summary = agent.plain_ask(messages).await?;
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("EXECUTION_SUMMARY: {execution_summary}");
        }

        let is_done_system_prompt = "Based on the next message, is our goal reached? return true if reached and false otherwise".to_string();
        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: is_done_system_prompt,
            },
            MessageRequest {
                role: MessageRole::User,
                content: execution_summary.clone(),
            },
        ];

        let is_done = agent.plain_ask(messages).await?;
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("IS_DONE? {is_done}");
        }

        Ok((is_done.to_lowercase() == "true", execution_summary))
    }

    async fn split_prompt(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<Vec<String>, Error> {
        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: task_splitter_prompt(),
            },
            MessageRequest {
                role: MessageRole::User,
                content: input,
            },
        ];

        let result = agent.plain_ask(messages).await?;
        Ok(result.split(";").map(|i| i.trim().to_string()).collect())
    }

    async fn is_prompt_complex(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<bool, Error> {
        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: is_prompt_complex_prompt(),
            },
            MessageRequest {
                role: MessageRole::User,
                content: input,
            },
        ];

        let result = agent.plain_ask(messages).await?;
        Ok(result == "true")
    }
}

#[derive(Debug)]
pub struct Step {
    prompt: String,
    result: Option<String>,
}

impl Step {
    pub fn new(prompt: String) -> Self {
        Self {
            prompt,
            result: None,
        }
    }

    #[inline]
    pub fn prompt(&self) -> String {
        self.prompt.clone()
    }

    #[inline]
    pub fn finish(&mut self, result: String) -> () {
        self.result = Some(result);
    }
}

fn task_splitter_prompt() -> String {
    load_file_installed("prompts/task_splitter.md", vec![])
}

fn is_prompt_complex_prompt() -> String {
    load_file_installed("prompts/is_prompt_complex.md", vec![])
}
