#! /bin/bash

# Execute auxiliar install script
bash $PWD/install_prompts_and_tools.sh

# Builds and install the project
cargo install --path .
