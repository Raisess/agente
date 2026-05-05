use crate::adapters::util::load_file::load;

/// Function to load a specific file
pub fn load_file_installed(filename: &str, replace: Vec<(&str, String)>) -> String {
    let content = load(filename, replace.clone()).unwrap_or_else(|_| {
        let path = format!("{}/.agente/{}", std::env!("HOME"), filename);
        load(&path, replace).expect(&format!("Failed to load the {filename} file"))
    });

    content
}
