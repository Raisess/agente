use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use agente_domain::error::Error;

static __CACHE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// This function loads a file content and store into the memory as a HashMap,
/// using the entire path as key.
pub fn load(path: &str, replace: Vec<(&str, String)>) -> Result<String, Error> {
    let mut binding = __CACHE.lock().unwrap();
    let Some(content) = binding.get(path) else {
        if !std::fs::exists(&path)? {
            return Err(Error::new(&format!(
                "File: {path} not found, can't procced."
            )));
        }

        let mut content = std::fs::read_to_string(&path)?;
        for (key, value) in replace {
            content = content.replace(&format!("{{{{{key}}}}}"), &value);
        }

        binding.insert(path.to_string(), content.clone());
        return Ok(content);
    };

    Ok(content.clone())
}
