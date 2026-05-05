#! /bin/bash

# Copy the tools to $HOME/.agente
mkdir $HOME/.agente
cp /home/lara/code/agente/tools.json $HOME/.agente/
cp -r /home/lara/code/agente/tools $HOME/.agente/
cp -r /home/lara/code/agente/prompts $HOME/.agente/

# Builds and install the project
cargo install --path .
