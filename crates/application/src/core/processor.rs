use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use agente_domain::error::Error;
use agente_domain::ports::ai_provider::{AiProvider, AskResponse};
use agente_domain::ports::io::{Executor, ExecutorArgument};
use agente_infrastructure::adapters::util::cmd::CMD;
use agente_infrastructure::adapters::util::load_file::load;
use agente_infrastructure::config::Config;

use crate::core::context::Context;

const MAX_COMMAND_OUTPUT_SIZE: usize = 1000;

// @TODO: link a message id, will be useful for websocket server to know message
// and tool contexts
pub enum TaskResponse {
    Done,
    Thinking,
    MessageResponse(String),
    CommandSignature(String),
    CommandResponse((String, String)),
    Error(Error),
}

pub struct ToolResponse {
    pub output: String,
    pub croped_output: String,
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
        let tasks = if prompt.len() > 300 || prompt.to_lowercase().contains("analyze") {
            vec![]
        } else {
            if self.is_prompt_complex(prompt.clone()).await? {
                self.split_prompt(prompt).await?
            } else {
                vec![prompt]
            }
        };

        for task in tasks {
            match self.recursively_process_prompt(task, None).await {
                Ok(_) => {}
                Err(error) => {
                    self.__sender.send(TaskResponse::Error(error)).await?;
                }
            }
        }

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
                        .send(TaskResponse::CommandSignature(tool.to_string()))
                        .await?;

                    match self.execute_tool(&tool, &arguments) {
                        Ok(tool_response) => {
                            self.__sender
                                .send(TaskResponse::CommandResponse((
                                    tool,
                                    tool_response.croped_output,
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

        self.__sender.send(TaskResponse::Done).await?;
        Ok(())
    }

    async fn process_prompt(&mut self, input: String) -> Result<AskResponse, Error> {
        self.context.summarize(&self.agent, false).await?;

        let response = self.context.ask(&self.agent, input).await?;
        Ok(response)
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
        Ok(if result == "true" { true } else { false })
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
        let mut croped_output = output.clone();
        croped_output.truncate(MAX_COMMAND_OUTPUT_SIZE);

        Ok(ToolResponse {
            output,
            croped_output,
            refeed: if tool != "write" { true } else { false },
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
    load("prompts/task_splitter.md", vec![]).expect("Failed to load task splitter prompt")
}

fn is_prompt_complex_prompt() -> String {
    load("prompts/is_prompt_complex.md", vec![])
        .expect("Failed to load is prompt complex prompt")
}
