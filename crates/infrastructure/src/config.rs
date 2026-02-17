use std::sync::Arc;

use serde::{Deserialize, Serialize};

use agente_domain::ports::io::{Reader, Writer};

/// Represents the agent settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Your agent name, fallbacks to `agente`
    pub name: Option<String>,
    /// plain Chat GPT api key
    pub chat_gpt_api_key: String,
    /// Optional system prompt to initialize the agent, fallback to default if
    /// not setted
    pub system_prompt_path: Option<String>,
    /// The prompt used by the context module to summarize the messages list
    pub summarizer_prompt_path: String,
}

impl Config {
    pub fn load(
        reader: Arc<dyn Reader>,
        path: Option<&str>,
    ) -> Result<Arc<Self>, std::io::Error> {
        let content =
            reader.read(path.unwrap_or(&default_config_path())).expect(
                "Failed to load config.json on the current path and from \
                 ~/.config/agente/config.json",
            );

        let mut config = serde_json::from_str::<Config>(&content)?;
        config.name = Some(config.name.unwrap_or("Agente".to_string()));
        config.system_prompt_path = Some(
            config
                .system_prompt_path
                .unwrap_or(default_system_prompt_path()),
        );

        Ok(Arc::new(config))
    }

    pub fn update(
        &self,
        writer: Arc<dyn Writer>,
        path: Option<&str>,
    ) -> Result<(), std::io::Error> {
        Ok(writer.write(
            path.unwrap_or(&default_config_path()),
            &serde_json::to_string(self)?.as_bytes(),
        )?)
    }

    pub fn pwd() -> Option<String> {
        std::env::current_dir()
            .expect("Failed to get current dir")
            .to_str()
            .map(String::from)
    }
}

fn default_system_prompt_path() -> String {
    String::from(format!("{}/prompts/system.md", config_folder_path()))
}

fn default_config_path() -> String {
    String::from(format!("{}/config.json", config_folder_path()))
}

fn config_folder_path() -> String {
    let home = std::env!("HOME");
    String::from(format!("{home}/.config/agente"))
}
