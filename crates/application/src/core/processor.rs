use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::debug;

use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, AskResponse};
use agente_domain::ports::io::{Executor, ExecutorArgument};
use agente_infrastructure::adapters::util::cmd::CMD;
use agente_infrastructure::config::Config;

use crate::core::context::Context;

static MAX_ERROR_RETRIES: usize = 3;

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
        match prompt.trim() {
            "/exit" => std::process::exit(0),
            "/compact" => {
                self.context.summarize(&self.agent, true).await?;
                self.__sender
                    .send(TaskResponse::MessageResponse(
                        "Conversation compacted!".to_string(),
                    ))
                    .await?;
            }
            v => self.mloop(v.to_string(), 0).await?,
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
        match self
            .recursively_process_prompt(prompt.clone(), false, None)
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

        Ok(())
    }

    #[async_recursion::async_recursion]
    async fn recursively_process_prompt(
        &mut self,
        prompt: String,
        is_refeed: bool,
        last_executed_tool_hash: Option<String>,
    ) -> Result<(), Error> {
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
            }
            AskResponse::ToolCall(tools) => {
                debug!(name: "tools", "tools that will be used: {tools:?}");

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

                    if tool_response.refeed {
                        self.recursively_process_prompt(
                            tool_response.output,
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
