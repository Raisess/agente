use std::collections::HashMap;

use agente_domain::core::command::Command;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, info};

use agente_domain::core::Error;
use agente_domain::core::models::task::Task;
use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::Agent;

use crate::context::Context;
use crate::prompt::system_prompt;

pub struct Runtime {
    agent: Box<dyn Agent>,
    tools: HashMap<String, Box<dyn Tool>>,
    commands: HashMap<String, Box<dyn Command>>,
    context: Context,
    last_feed_response: String,
}

impl Runtime {
    pub fn init(
        agent: Box<dyn Agent>,
        tools: HashMap<String, Box<dyn Tool>>,
        commands: HashMap<String, Box<dyn Command>>,
    ) -> Self {
        Self {
            agent,
            commands,
            context: Context::init(system_prompt(&tools)),
            last_feed_response: String::new(),
            tools,
        }
    }

    pub async fn run(
        &mut self,
        input_rx: &mut Receiver<String>,
        output_tx: Sender<String>,
    ) -> () {
        while let Some(input) = input_rx.recv().await {
            let input = input.trim().to_string();
            if input.starts_with("/") {
                self.process_command(input);
            } else {
                self.process_execution_plan(&output_tx, input).await;
            }
        }
    }

    fn process_command(&mut self, input: String) -> () {
        if let Some(command) = self.commands.get(&input.clone().split_off(1)) {
            return command.execute().expect("Failed to execute command");
        } else {
            println!("Command not found for {input}!");
            return ();
        }
    }

    async fn process_execution_plan(
        &mut self,
        output_tx: &Sender<String>,
        input: String,
    ) -> () {
        println!("Thinking...");
        let execution_plan = self
            .context
            .ask(&mut self.agent, input)
            .await
            .expect("Failed to ask the agent for the execution plan");
        let execution_plan_len = execution_plan.len();

        for task in execution_plan {
            match self
                .process_task(
                    task,
                    execution_plan_len,
                    self.last_feed_response.clone(),
                )
                .await
            {
                Ok(response) => {
                    self.last_feed_response = response;
                    output_tx
                        .send(self.last_feed_response.clone())
                        .await
                        .expect("Failed to send response to output thread");
                }
                Err(error) => {
                    error!("Failed to process task: {}", error.message())
                }
            }
        }

        self.context
            .summarize(&mut self.agent)
            .await
            .expect("Failed to summarize messages");
    }

    async fn process_task(
        &mut self,
        task: Task,
        execution_plan_len: usize,
        last_feed_response: String,
    ) -> Result<String, Error> {
        let key = task.tool();
        let tool = self
            .tools
            .get(&key)
            .expect(&format!("Tool not found: {key}"));

        let mut args = task.arguments();
        // @NOTE: add the last feed response as argument if the task is part of
        // a execution plan
        if execution_plan_len > 1 {
            args.push(last_feed_response);
        }

        let result = tool.handle(args).await?;
        info!(name: "tool_result", "{key}: {result:#?}");
        if result.is_feedable && !result.data.is_empty() {
            let mut message = result.data;
            if let Some(usage) = tool.usage_instruction() {
                message = format!("{usage}: {message}");
            }

            let response =
                self.context.feed(&mut self.agent, message).await.expect(
                    "Failed to feed agent with tool result information",
                );
            return Ok(response);
        } else {
            return Ok(result.data);
        }
    }
}
