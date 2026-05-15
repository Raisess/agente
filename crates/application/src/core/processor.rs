use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, AskResponse};
use agente_domain::ports::io::{Executor, ExecutorArgument};
use agente_infrastructure::adapters::util::cmd::CMD;
use agente_infrastructure::adapters::util::load_file_installed::load_file_installed;
use agente_infrastructure::config::Config;

use crate::core::context::Context;

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
            _ => {
                let tasks = self.refine_prompt(prompt).await?;
                if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
                    println!("TASKS: {:#?}", tasks);
                }

                for task in tasks {
                    match self.recursively_process_prompt(task, None).await {
                        Ok(_) => {}
                        Err(error) => {
                            self.__sender.send(TaskResponse::Error(error)).await?;
                        }
                    }
                }
            }
        }

        self.__sender.send(TaskResponse::Done).await?;
        Ok(())
    }

    #[async_recursion::async_recursion]
    async fn recursively_process_prompt(
        &mut self,
        prompt: String,
        last_executed_tool_hash: Option<String>,
    ) -> Result<(), Error> {
        if prompt.is_empty() {
            return Ok(());
        }

        self.__sender.send(TaskResponse::Thinking).await?;
        let response = self.process_prompt(prompt.clone()).await?;
        match response {
            AskResponse::Content(text) => {
                self.__sender
                    .send(TaskResponse::MessageResponse(text))
                    .await?;
            }
            AskResponse::ToolCall(tools) => {
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

                    match self.execute_tool(&tool, &arguments) {
                        Ok(tool_response) => {
                            self.__sender
                                .send(TaskResponse::ToolResponse((
                                    tool,
                                    arguments,
                                    tool_response.output.clone(),
                                )))
                                .await?;

                            if tool_response.refeed {
                                self.recursively_process_prompt(
                                    tool_response.output,
                                    Some(hash),
                                )
                                .await?;
                            }
                        }
                        Err(e) => {
                            self.__sender.send(TaskResponse::Error(e)).await?;

                            let failed_prompt = format!(
                                "Failed to process prompt: {prompt}, use another tool \
                                 to find context and then retry it"
                            );
                            self.recursively_process_prompt(failed_prompt, Some(hash))
                                .await?;
                        }
                    };
                }
            }
        }

        Ok(())
    }

    async fn process_prompt(&mut self, input: String) -> Result<AskResponse, Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input).await?;
        Ok(response)
    }

    async fn refine_prompt(&self, input: String) -> Result<Vec<String>, Error> {
        if input.len() < 20 {
            Ok(vec![input])
        } else if self.is_prompt_complex(input.clone()).await? {
            Ok(self.split_prompt(input).await?)
        } else {
            Ok(vec![input])
        }
    }

    async fn split_prompt(&self, input: String) -> Result<Vec<String>, Error> {
        let result = self.agent.plain_ask(task_splitter_prompt(), input).await?;
        Ok(result.split(";").map(|i| i.trim().to_string()).collect())
    }

    async fn is_prompt_complex(&self, input: String) -> Result<bool, Error> {
        let result = self
            .agent
            .plain_ask(is_prompt_complex_prompt(), input)
            .await?;
        if std::env::var("DEBUG_PROMPT").unwrap_or("0".to_string()) == "1" {
            println!("IS_COMPLEX: {result}");
        }

        Ok(result == "true")
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

fn task_splitter_prompt() -> String {
    load_file_installed("prompts/task_splitter.md", vec![])
}

fn is_prompt_complex_prompt() -> String {
    load_file_installed("prompts/is_prompt_complex.md", vec![])
}
