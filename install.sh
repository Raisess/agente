#! /bin/bash

# Copy the tools to $HOME/.agente
mkdir $HOME/.agente
cp $PWD/tools.json $HOME/.agente/
cp -r $PWD/tools $HOME/.agente/
cp -r $PWD/prompts $HOME/.agente/

# Builds and install the project
cargo install --path .
