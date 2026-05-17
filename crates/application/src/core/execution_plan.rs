use agente_domain::error::Error;
use agente_domain::ports::ai_provider::AiProvider;
use agente_infrastructure::adapters::util::load_file_installed::load_file_installed;

#[derive(Debug)]
pub struct ExecutionPlan {
    pub is_complex: bool,
    pub is_done: bool,
    pub size: usize,
    pub steps: Vec<String>,
}

impl ExecutionPlan {
    pub async fn generate(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<Self, Error> {
        let (steps, is_complex) = if input.len() < 20 {
            (vec![input], false)
        } else if Self::is_prompt_complex(agent, input.clone()).await? {
            // @TODO: should we reasoning this prompt??
            (Self::split_prompt(agent, input).await?, true)
        } else {
            (vec![input], false)
        };

        let plan = Self {
            is_complex,
            is_done: false,
            size: steps.len(),
            steps,
        };
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("PLAN: {:#?}", plan);
        }

        Ok(plan)
    }

    async fn split_prompt(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<Vec<String>, Error> {
        let result = agent.plain_ask(task_splitter_prompt(), input).await?;
        Ok(result.split(";").map(|i| i.trim().to_string()).collect())
    }

    async fn is_prompt_complex(
        agent: &Box<dyn AiProvider>,
        input: String,
    ) -> Result<bool, Error> {
        let result = agent.plain_ask(is_prompt_complex_prompt(), input).await?;
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("IS_COMPLEX: {result}");
        }

        Ok(result == "true")
    }
}

fn task_splitter_prompt() -> String {
    load_file_installed("prompts/task_splitter.md", vec![])
}

fn is_prompt_complex_prompt() -> String {
    load_file_installed("prompts/is_prompt_complex.md", vec![])
}
