use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, AskResponse};
use agente_domain::ports::io::{Executor, ExecutorArgument};
use agente_infrastructure::adapters::util::cmd::CMD;
use agente_infrastructure::config::Config;

use crate::core::context::Context;
use crate::core::execution_plan::{ExecutionPlan, Step};

static MAX_ERROR_RETRIES: usize = 3;

// @TODO: link a message id, will be useful for websocket server to know message
// and tool contexts
pub enum TaskResponse {
    Done,
    Thinking,
    MessageResponse(String),
    ToolSignature((String, HashMap<String, String>)),
    ToolResponse((String, HashMap<String, String>, String)),
    Error(Error),
}

pub struct ToolResponse {
    pub output: String,
    pub refeed: bool,
}

pub struct Processor {
    __receiver: Arc<Mutex<Receiver<TaskResponse>>>,
    __sender: Sender<TaskResponse>,
    agent: Box<dyn AiProvider>,
    context: Context,
    cmd: CMD,
}

impl Processor {
    pub fn init(agent: Box<dyn AiProvider>, context: Context) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<TaskResponse>(10);
        Self {
            __receiver: Arc::new(Mutex::new(rx)),
            __sender: tx,
            agent,
            context,
            cmd: CMD::default(),
        }
    }

    pub fn listener(&self) -> Arc<Mutex<Receiver<TaskResponse>>> {
        self.__receiver.clone()
    }

    pub async fn handle(&mut self, prompt: String) -> Result<(), Error> {
        match prompt.as_str() {
            "/exit" => std::process::exit(0),
            "/compact" => {
                self.context.summarize(&self.agent, true).await?;
                self.__sender
                    .send(TaskResponse::MessageResponse(
                        "Conversation compacted!".to_string(),
                    ))
                    .await?;
            }
            _ => self.mloop(prompt, 0).await?,
        }

        self.__sender.send(TaskResponse::Done).await?;
        Ok(())
    }

    #[async_recursion::async_recursion]
    async fn mloop(
        &mut self,
        prompt: String,
        mut max_retries: usize,
    ) -> Result<(), Error> {
        let mut plan = ExecutionPlan::generate(&self.agent, prompt.clone()).await?;
        for step in &mut plan.steps {
            match self
                .recursively_process_prompt(&mut *step, false, None)
                .await
            {
                Ok(_) => {}
                Err(error) => {
                    max_retries += 1;
                    if max_retries >= MAX_ERROR_RETRIES {
                        return Err(error);
                    }

                    self.__sender.send(TaskResponse::Error(error)).await?;
                    let failed_prompt = format!(
                        "Failed to process prompt: {prompt}, use another tool \
                                 to find context and then retry it"
                    );
                    self.mloop(failed_prompt, max_retries).await?;
                }
            }
        }

        let (is_done, final_result) = plan.is_done(&self.agent).await?;
        if !is_done {
            self.mloop(final_result, max_retries).await?;
        }

        Ok(())
    }

    #[async_recursion::async_recursion]
    async fn recursively_process_prompt(
        &mut self,
        step: &mut Step,
        is_refeed: bool,
        last_executed_tool_hash: Option<String>,
    ) -> Result<(), Error> {
        let prompt = step.prompt();
        if prompt.is_empty() {
            return Ok(());
        }

        self.__sender.send(TaskResponse::Thinking).await?;
        let response = self.process_prompt(prompt.clone(), is_refeed).await?;
        match response {
            AskResponse::Content(text) => {
                self.__sender
                    .send(TaskResponse::MessageResponse(text.clone()))
                    .await?;
                step.finish(text);
            }
            AskResponse::ToolCall(tools) => {
                if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
                    println!("TOOLS: {:#?}", tools);
                }

                for (tool, arguments) in tools {
                    let hash = Self::generate_tool_hash(&tool, &arguments);
                    if last_executed_tool_hash.is_some()
                        && last_executed_tool_hash.eq(&Some(hash.clone()))
                    {
                        let error = Error::new(&format!(
                            "Trying to execute the same tool again: {hash}"
                        ));
                        return Err(error);
                    }

                    self.__sender
                        .send(TaskResponse::ToolSignature((
                            tool.to_string(),
                            arguments.clone(),
                        )))
                        .await?;

                    let tool_response = self.execute_tool(&tool, &arguments)?;
                    self.__sender
                        .send(TaskResponse::ToolResponse((
                            tool.clone(),
                            arguments,
                            tool_response.output.clone(),
                        )))
                        .await?;
                    step.finish(tool_response.output.clone());

                    if tool_response.refeed {
                        let mut tool_response_step = Step::new(tool_response.output);
                        self.recursively_process_prompt(
                            &mut tool_response_step,
                            true,
                            Some(hash),
                        )
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_prompt(
        &mut self,
        input: String,
        is_refeed: bool,
    ) -> Result<AskResponse, Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input, is_refeed).await?;
        Ok(response)
    }

    fn execute_tool(
        &self,
        tool: &String,
        arguments: &HashMap<String, String>,
    ) -> Result<ToolResponse, Error> {
        let tools_path = Config::default_tools_path();
        let mut script = vec![ExecutorArgument::Arg(format!("{tools_path}/{tool}.py"))];
        let mut flags = arguments
            .iter()
            .map(|(key, value)| ExecutorArgument::Flag((key.clone(), value.clone())))
            .collect::<Vec<_>>();

        script.append(&mut flags);

        let output = self.cmd.exec(
            "python3",
            script,
            vec![("WORKING_DIR".to_string(), Config::pwd())],
        )?;

        Ok(ToolResponse {
            output,
            refeed: tool != "write",
        })
    }

    fn generate_tool_hash(tool: &String, arguments: &HashMap<String, String>) -> String {
        format!(
            "{tool} {}",
            arguments
                .iter()
                .map(|(key, value)| format!("{key}: {value}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
