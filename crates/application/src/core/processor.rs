use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::info;

use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, AskResponse};
use agente_domain::ports::io::{Executor, ExecutorArgument};
use agente_infrastructure::adapters::util::cmd::CMD;
use agente_infrastructure::adapters::util::file_system::FileSystem;
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
    pub(super) agent: Box<dyn AiProvider>,
    pub(super) context: Context,
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

    /// Exposes the receveiver mutex for processor internal events handling
    /// by other parts of the application.
    pub fn listener(&self) -> Arc<Mutex<Receiver<TaskResponse>>> {
        self.__receiver.clone()
    }

    /// This method sends a event that can be used by the stdio interface to
    /// show up the input prompt again.
    pub async fn allow_prompt(&self) -> Result<(), Error> {
        self.__sender.send(TaskResponse::Done).await?;
        Ok(())
    }

    /// Process a prompt, this method will send events through the receiver
    /// (exposed by listener method).
    pub async fn handle(&mut self, prompt: String) -> Result<(usize, usize), Error> {
        let (start, end) = {
            let start = self.context.messages.len(); // not offseting will skip system prompt
            match prompt.trim() {
                "/exit" => self.exit().await?,
                "/compact" => self.compact().await?,
                "/dump" => self.dump().await?,
                v => self.prompt_retry_loop(v.to_string(), 0).await?,
            }
            let end = self.context.messages.len();
            (start, end)
        };

        Ok((start, end))
    }

    /// Summarize and exit the process with status code 0
    pub async fn exit(&mut self) -> Result<(), Error> {
        println!("Exiting... | Conversation compacting and saving...");
        self.context.summarize(&self.agent, true).await?;
        println!(">>> Your session id is: {}", self.context.session_id());
        std::process::exit(0);
    }

    async fn compact(&mut self) -> Result<(), Error> {
        self.context.summarize(&self.agent, true).await?;
        self.__sender
            .send(TaskResponse::MessageResponse(
                "Conversation compacted!".to_string(),
            ))
            .await?;
        Ok(())
    }

    async fn dump(&self) -> Result<(), Error> {
        let file_name = self.context.dump(FileSystem::default())?;
        let message = format!("Dump completed to {file_name}!");
        self.__sender
            .send(TaskResponse::MessageResponse(message))
            .await?;
        Ok(())
    }

    #[async_recursion::async_recursion]
    async fn prompt_retry_loop(
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
                self.prompt_retry_loop(prompt, max_retries).await?;
            }
        }

        Ok(())
    }

    // @TODO: if a tool execute and then fail to process the result, use the result
    // as the prompt to retry and dont reexecute the tool unless the tool fail
    #[async_recursion::async_recursion]
    async fn recursively_process_prompt(
        &mut self,
        prompt: String,
        is_tool_execution_result: bool,
        last_executed_tool_hash: Option<String>,
    ) -> Result<(), Error> {
        if prompt.is_empty() {
            return Ok(());
        }

        self.__sender.send(TaskResponse::Thinking).await?;
        let response = self
            .process_prompt(prompt.clone(), is_tool_execution_result)
            .await?;
        match response {
            AskResponse::Content(text) => {
                self.__sender
                    .send(TaskResponse::MessageResponse(text.clone()))
                    .await?;
            }
            AskResponse::ToolCall(tools) => {
                info!(name: "tools", "tools that will be used: {tools:?}");

                for (tool, arguments) in tools {
                    let hash = AskResponse::generate_tool_hash(&tool, &arguments);
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

        // @TODO: check input length and split it when it exceeds maximum allowed size
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
            vec![
                ("WORKING_DIR", Config::pwd()),
                ("MAX_RESULTS", Config::max_search_tool_results()),
            ],
        )?;

        Ok(ToolResponse {
            output,
            refeed: tool != "write",
        })
    }
}
