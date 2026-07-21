#! /usr/bin/env python3

import argparse
import shlex
import subprocess

NOT_ALLOWED_COMMANDS = ["sudo", "rm -rf /"]

def bash(command: str) -> None:
  for not_allowed_command in NOT_ALLOWED_COMMANDS:
    if not_allowed_command in command:
      print("Invalid command, can't execute:", ", ".join(NOT_ALLOWED_COMMANDS))
      return

  parsed_command = shlex.split(command)

  to_pop = []
  envs = {}
  for (i, part) in enumerate(parsed_command):
    if "=" in part:
      to_pop.append(i)
      (key, value) = part.split("=")
      envs[key] = value


  parsed_command = [v for i, v in enumerate(parsed_command) if i not in to_pop]
  p = subprocess.Popen(
    parsed_command,
    env=envs,
    stdout=subprocess.PIPE,
    stderr=subprocess.STDOUT,
    encoding="utf-8",
    errors="replace"
  )

  for line in p.stdout:
    print(line, end="")

  p.wait()

  if p.returncode != 0:
    print("\nProcess exited with code:", p.returncode)


if __name__ == "__main__":
  parser = argparse.ArgumentParser(description="Execute a bash command in the host machine")
  parser.add_argument("--command", type=str, help="Command to be executed")
  args = parser.parse_args()
  
  bash(args.command)
