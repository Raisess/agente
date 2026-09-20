use std::io::ErrorKind;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use agente_domain::ports::ai_provider::AiProviderConfig;
use agente_domain::ports::io::{Reader, Writer};

/// Represents the agent settings
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Agente alias name
    pub name: Option<String>,
    /// OpenAI config
    pub openai: Option<AiProviderConfig>,
    /// Groq config
    pub groq: Option<AiProviderConfig>,
    // @TODO: add openrouter
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

    pub fn setup_fallback<Fs>(
        fs: Arc<Fs>,
        setup_config: Option<Config>,
    ) -> Result<Arc<Self>, std::io::Error>
    where
        Fs: Reader + Writer + 'static,
    {
        let config_folder_path = config_folder_path();
        Self::create_dir(&config_folder_path)?;

        let json = if setup_config.is_some() {
            serde_json::to_string(&setup_config.unwrap())?
        } else {
            serde_json::to_string(&Config::default())?
        };

        fs.write(&default_config_path(), &json.as_bytes())?;
        Self::load(fs, None)
    }

    pub fn use_preprocessor() -> bool {
        std::env::var("USE_PREPROCESSOR")
            .unwrap_or("0".to_string())
            .eq("1")
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

    pub fn max_search_tool_results() -> String {
        std::env::var("MAX_SEARCH_TOOL_RESULTS").unwrap_or("10".to_string())
    }

    pub fn max_context_memory_size() -> usize {
        std::env::var("MAX_C_SIZE")
            .unwrap_or("356".to_string())
            .parse()
            .unwrap()
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

#[derive(Default)]
pub struct ConfigBuilder {
    name: String,
    provider: String,
    api_key: String,
    model: String,
    cheap_model: Option<String>,
}

impl ConfigBuilder {
    pub fn name(&mut self, name: String) -> &mut ConfigBuilder {
        self.name = name;
        self
    }

    pub fn provider(&mut self, provider: String) -> &mut ConfigBuilder {
        self.provider = provider;
        self
    }

    pub fn api_key(&mut self, api_key: String) -> &mut ConfigBuilder {
        self.api_key = api_key;
        self
    }

    pub fn model(&mut self, model: String) -> &mut ConfigBuilder {
        self.model = model;
        self
    }

    pub fn cheap_model(&mut self, cheap_model: Option<String>) -> &mut ConfigBuilder {
        self.cheap_model = cheap_model;
        self
    }

    pub fn build(&self) -> Config {
        let mut c = Config {
            name: Some(self.name.clone()),
            openai: None,
            groq: None,
        };

        match self.provider.as_str() {
            "openai" => {
                c.openai = Some(AiProviderConfig {
                    api_key: self.api_key.clone(),
                    model: self.model.clone(),
                    cheap_model: self.cheap_model.clone(),
                });
            }
            _ => panic!("Invalid provider for building config"),
        }

        c
    }
}
