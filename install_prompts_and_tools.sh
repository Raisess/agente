#! /bin/bash

# Copy the tools to $HOME/.agente
mkdir $HOME/.agente
cp $PWD/tools.json $HOME/.agente/
cp -r $PWD/tools $HOME/.agente/
cp -r $PWD/prompts $HOME/.agente/

python3 -m pip install -r ./tools/requirements.txt --break-system-packages
