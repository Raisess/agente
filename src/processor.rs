use std::collections::HashMap;

use agente_domain::core::command::Command;
use tracing::{error, info};

use agente_domain::core::Error;
use agente_domain::core::models::task::Task;
use agente_domain::core::tool::Tool;
use agente_domain::ports::agent::Agent;

use crate::context::Context;

pub struct Processor {
    agent: Box<dyn Agent>,
    tools: HashMap<String, Box<dyn Tool>>,
    commands: HashMap<String, Box<dyn Command>>,
    context: Context,
}

impl Processor {
    pub fn init(
        agent: Box<dyn Agent>,
        tools: HashMap<String, Box<dyn Tool>>,
        commands: HashMap<String, Box<dyn Command>>,
    ) -> Self {
        Self {
            agent,
            commands,
            context: Context::init(system_prompt(&tools)),
            tools,
        }
    }

    pub async fn handle(&mut self, input: String) -> Option<Vec<String>> {
        if input.starts_with("/") {
            return self.process_command(input);
        }

        return self.process_execution_plan(input).await;
    }

    async fn process_execution_plan(
        &mut self,
        input: String,
    ) -> Option<Vec<String>> {
        self.context
            .summarize(&mut self.agent)
            .await
            .expect("Failed to summarize messages");

        info!("asking...");
        let execution_plan = self
            .context
            .ask(&mut self.agent, input)
            .await
            .expect("Failed to ask the agent for the execution plan");

        let mut output = Vec::<String>::new();
        for task in execution_plan {
            let argument_from_context = output.last();
            match self.process_task(task, argument_from_context).await {
                Ok(response) => output.push(response),
                Err(error) => {
                    error!("Failed to process task: {}", error.message())
                }
            }
        }

        Some(output)
    }

    async fn process_task(
        &mut self,
        task: Task,
        argument_from_context: Option<&String>,
    ) -> Result<String, Error> {
        let key = task.tool();
        let tool = self
            .tools
            .get(&key)
            .expect(&format!("Tool not found: {key}"));

        let mut args = task.arguments();
        // @NOTE: add the last feed response as argument if the task is part of
        // a execution plan
        match argument_from_context {
            Some(argument) => args.push(argument.clone()),
            None => {}
        }

        let result = tool.handle(args).await?;
        info!(name: "tool_result", "{key}: {result:#?}");
        if result.is_feedable && !result.data.is_empty() {
            let mut message = result.data;
            if let Some(usage) = tool.usage_instruction() {
                message = format!("<usage>{usage}</usage> {message}");
            }

            message =
                format!("<summary>{}</summary> {message}", task.summary());
            let response =
                self.context.feed(&mut self.agent, message).await.expect(
                    "Failed to feed agent with tool result information",
                );
            return Ok(response);
        } else {
            return Ok(result.data);
        }
    }

    fn process_command(&mut self, input: String) -> Option<Vec<String>> {
        if let Some(command) = self.commands.get(&input.clone().split_off(1)) {
            command.execute().expect("Failed to execute command");
        } else {
            println!("Command not found for {input}!");
        }

        None
    }
}

use agente_application::prompt::load;

fn system_prompt(tools: &HashMap<String, Box<dyn Tool>>) -> String {
    let tools_prompt = tools
        .iter()
        .map(|(name, tool)| {
            format!(
                "{name}(context: \"{}\", arguments format: \"{}\")",
                tool.context(),
                tool.format_instruction().unwrap_or("[]")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    load("system", vec![("tools", tools_prompt)])
        .expect("Failed to load system prompt")
}
