use std::{io::ErrorKind, sync::Arc};

use serde::{Deserialize, Serialize};

use agente_domain::ports::io::{Reader, Writer};

/// Represents the agent settings
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
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
        let content = reader.read(path.unwrap_or(&default_config_path()))?;

        let mut config = serde_json::from_str::<Config>(&content)?;
        config.name = Some(config.name.unwrap_or("Agente".to_string()));
        // @FIXME: load system prompt from current path if the exists
        config.system_prompt_path = Some(
            config
                .system_prompt_path
                .unwrap_or(default_system_prompt_path()),
        );
        config.summarizer_prompt_path = String::from("__prompts/summarizer.md");

        Ok(Arc::new(config))
    }

    pub fn setup_fallback<Fs>(fs: Arc<Fs>) -> Result<Arc<Self>, std::io::Error>
    where
        Fs: Reader + Writer + 'static,
    {
        let config_folder_path = config_folder_path();
        Self::create_dir(&config_folder_path)?;

        fs.write(
            &default_config_path(),
            &serde_json::to_string(&Config::default())?.as_bytes(),
        )?;
        Self::load(fs, None)
    }

    pub fn pwd() -> Option<String> {
        std::env::current_dir()
            .expect("Failed to get current dir")
            .to_str()
            .map(String::from)
    }

    fn create_dir(path: &str) -> Result<(), std::io::Error> {
        match std::fs::create_dir(path) {
            Ok(_) => {}
            Err(error) => {
                if error.kind() != ErrorKind::AlreadyExists {
                    return Err(error);
                }
            }
        }

        Ok(())
    }
}

fn default_system_prompt_path() -> String {
    String::from("__prompts/system.md")
}

fn default_config_path() -> String {
    format!("{}/config.json", config_folder_path())
}

fn config_folder_path() -> String {
    let home = std::env!("HOME");
    format!("{home}/.config/agente")
}
