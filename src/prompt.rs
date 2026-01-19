use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

const PROMPTS_FOLDER_PATH: &str = "__prompts";

static __CACHE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// This function loads a markdown file and store into the memory as a HashMap,
/// using the entire path as key.
///
/// @param name - is the file name without the `__prompts` folder prefix, if it
/// is inside a folder into `__prompts`, use/// like this: `folder/name`,
/// remeber to not use the file extension.
pub fn load(
    name: &str,
    replace: Vec<(&str, String)>,
) -> Result<String, std::io::Error> {
    let path = format!("{PROMPTS_FOLDER_PATH}/{name}.md");

    let mut binding = __CACHE.lock().unwrap();
    let Some(content) = binding.get(&path) else {
        if !std::fs::exists(&path)? {
            panic!("File: {path} not found, can't procced.");
        }

        let mut content = std::fs::read_to_string(&path)?;

        for (key, value) in replace {
            content = content.replace(&format!("{{{{{key}}}}}"), &value);
        }

        binding.insert(path, content.clone());
        return Ok(content);
    };

    Ok(content.clone())
}
