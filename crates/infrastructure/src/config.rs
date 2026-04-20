use std::io::ErrorKind;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use agente_domain::ports::io::{Reader, Writer};

use crate::adapters::providers::chat_gpt::ChatGPTConfig;

/// Represents the agent settings
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Chat GPT config
    #[serde(flatten)]
    pub chat_gpt: ChatGPTConfig,
}

impl Config {
    pub fn load(
        reader: Arc<dyn Reader>,
        path: Option<&str>,
    ) -> Result<Arc<Self>, std::io::Error> {
        let content = reader.read(path.unwrap_or(&default_config_path()))?;
        let config = serde_json::from_str::<Config>(&content)?;
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

    pub fn pwd() -> String {
        std::env::current_dir()
            .expect("Failed to get current dir")
            .to_str()
            .map(String::from)
            .expect("Can't get current directory")
    }

    pub fn port() -> String {
        std::env::var("PORT").unwrap_or("3030".to_string())
    }

    pub fn db_file() -> String {
        format!("{}/sqlite.db", config_folder_path())
    }

    pub fn default_tools_path() -> String {
        let local_path = "./tools".to_string();
        if std::fs::exists(&local_path).expect("Can't confirm if local tools path exists")
        {
            return local_path;
        }

        let path = format!("{}/tools", installed_folder_path());
        if std::fs::exists(&path).expect("Can't confirm if local tools path exists") {
            return path;
        }

        panic!("No tools folder match")
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

fn default_config_path() -> String {
    format!("{}/config.json", config_folder_path())
}

fn config_folder_path() -> String {
    let home = std::env!("HOME");
    format!("{home}/.config/agente")
}

fn installed_folder_path() -> String {
    let home = std::env!("HOME");
    format!("{home}/.agente")
}
