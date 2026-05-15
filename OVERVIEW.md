### Project Structure and Components:

#### Main Project Directory:
- `agente`
  - `Cargo.toml`: The main configuration file for the project.
  - `rustfmt.toml`: Configuration file for code formatting in Rust.
  - `LICENSE`: The license file for the project.
  - `README.md`: Readme file containing general information about the project.

#### Source Code:
- `agente/src`
  - `ansi.rs`: Rust code file related to handling ANSI sequences.
  - `websocket.rs`: Rust code file related to WebSocket functionalities.
  - `main.rs`: Main Rust code file for the project.
  - `stdio.rs`: Rust code file for handling standard input and output.
  - `lib.rs`: Rust library file.

#### Prompts:
- `agente/prompts`
  - `system.md`: Markdown file containing system-related prompts.
  - `task_splitter.md`: Markdown file with prompts related to task splitting.
  - `summarizer.md`: Markdown file containing prompts for summarization.
  - `is_prompt_complex.md`: Markdown file with prompts related to complexity identification.

#### Tools:
- `agente/tools`
  - `search.py`: Python script for searching information.
  - `read.py`: Python script for reading files.
  - `write.py`: Python script for writing content.
  - `fetch.py`: Python script for fetching data from URLs.
  - `explore.py`: Python script for exploring the project directory.
  - `bash.py`: Python script for executing bash commands.
  - `requirements.txt`: File containing project dependencies.
  - `__common.py`: Common functions used by other tools.

#### Crates (Rust Packages):
- `agente/crates`
  - `infrastructure`: Rust package for infrastructure-related code.
  - `domain`: Rust package for domain-specific code.
  - `application`: Rust package for application logic.

#### Crates Structure:
- `infrastructure`: Handles infrastructure-related functionalities.
- `domain`: Contains domain models, errors, and ports for communication.
- `application`: Implements the core logic of the application.

#### Additional Files:
- `install.sh`: Shell script for installation.
- `Cargo.lock`: Rust project lock file.

### Functionalities and Usage Guide:
1. **Running the Project:**
   - Use `Cargo run` to compile and run the main project.
  
2. **Using Tools:**
   - Utilize the provided Python scripts in the `tools` directory for various tasks.
  
3. **Development with Crates:**
   - Utilize the `infrastructure`, `domain`, and `application` crates for different aspects of the application.
  
4. **Additional Files:**
   - Refer to `README.md` for general project information.
   - Use `LICENSE` for licensing details.

### Instructions for Usage:
1. Clone the project repository.
2. Navigate to the project directory.
3. Install tools dependencies `python3 -m pip install -r ./tools/requirements.txt --break-system-packages`.
4. Run the project with `cargo run`.
5. Utilize the tools in the `tools` directory for various tasks.
6. Refer to specific directories and files for more detailed information.
