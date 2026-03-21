#! /usr/bin/env python3

import subprocess
import argparse
import os

FOLDERS_TO_IGNORE = [".git", "node_modules", "target", "build", "dist", ".next", ".cache", "__pycache__", "venv"]

WORKING_DIR = os.getenv("WORKING_DIR", None)

def find_command(path: str, ignore_list: list[str]) -> str:
  if not path.startswith(".") and not WORKING_DIR in path:
    raise Exception(f"Invalid path, outside working dir: {WORKING_DIR}")

  cmd = ["find", path]

  for ignore in ignore_list:
    cmd += ["!", "-path", f"*/{ignore}*"]

  result = subprocess.run(
    cmd,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
  )

  if result.returncode != 0:
    print("Error:", result.stderr)

  return result.stdout.strip()


def main():
  parser = argparse.ArgumentParser(
    description="Run find while ignoring specific folders"
  )
  parser.add_argument(
    "--path",
    type=str,
    help="Directory to search"
  )

  args = parser.parse_args()
  output = find_command(args.path, FOLDERS_TO_IGNORE)
  print(output)


if __name__ == "__main__":
    main()
