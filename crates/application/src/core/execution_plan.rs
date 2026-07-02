use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, MessageRequest, MessageRole};
use agente_infrastructure::adapters::util::load_file_installed::load_file_installed;

#[derive(Debug)]
pub struct ExecutionPlan {
    pub steps: Vec<Step>,
    pub size: usize,
    pub is_complex: bool,
    last_execution_summary: Option<String>,
}

// @TODO: check if each step is done individaully and stop mloop in processor
// @TODO: store the original input and determine if its done based on the step results
impl ExecutionPlan {
    pub async fn generate(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<Self, Error> {
        let (steps, is_complex) = if Self::is_prompt_complex(agent, input.clone()).await?
        {
            let results = Self::split_prompt(agent, input).await?;
            let steps = results.iter().map(|r| Step::new(r.to_string())).collect();
            (steps, true)
        } else {
            (
                vec![Step {
                    prompt: input.clone(),
                    result: None,
                }],
                false,
            )
        };

        let plan = Self {
            size: steps.len(),
            steps,
            is_complex,
            last_execution_summary: None,
        };
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("PLAN: {:#?}", plan);
        }

        Ok(plan)
    }

    pub async fn is_done(
        &mut self,
        agent: &Box<dyn AiProvider>,
    ) -> Result<(bool, String), Error> {
        let executed_plan_summary = self
            .steps
            .iter()
            .map(|step| {
                format!(
                    "Request: '{}' | Response: '{}'",
                    step.prompt,
                    step.result.clone().unwrap_or("Not finished".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let messages = vec![
            MessageRequest {
                role: MessageRole::System,
                content: confirm_execution_plan_is_done(),
            },
            MessageRequest {
                role: MessageRole::User,
                content: format!("What is executed: {executed_plan_summary}"),
            },
        ];

        let is_done = agent.plain_ask(messages).await?;
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("IS_DONE? {is_done} SUMMARY: {executed_plan_summary:#?}");
        }

        if is_done.to_lowercase().contains("true") {
            return Ok((true, "".to_string()));
        }

        let mut messages = vec![MessageRequest {
            role: MessageRole::System,
            content: generate_new_prompt_when_not_done(),
        }];

        if self.last_execution_summary.is_some() {
            let summary = self.last_execution_summary.clone().unwrap();
            let content = format!("This is the last execution summary: {summary}");
            messages.push(MessageRequest {
                role: MessageRole::User,
                content,
            });
        }

        messages.push(MessageRequest {
            role: MessageRole::User,
            content: executed_plan_summary.clone(),
        });

        let new_prompt = agent.plain_ask(messages).await?;
        self.last_execution_summary = Some(executed_plan_summary);
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("EXECUTION_SUMMARY: {new_prompt}");
        }

        Ok((false, new_prompt))
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

fn confirm_execution_plan_is_done() -> String {
    load_file_installed(
        "prompts/execution_plan/confirm_execution_plan_is_done.md",
        vec![],
    )
}

fn generate_new_prompt_when_not_done() -> String {
    load_file_installed(
        "prompts/execution_plan/generate_new_prompt_when_not_done.md",
        vec![],
    )
}

fn task_splitter_prompt() -> String {
    load_file_installed("prompts/execution_plan/task_splitter.md", vec![])
}

fn is_prompt_complex_prompt() -> String {
    load_file_installed("prompts/execution_plan/is_prompt_complex.md", vec![])
}
