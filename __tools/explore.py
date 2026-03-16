#! /usr/bin/env python3

import subprocess
import argparse

FOLDERS_TO_IGNORE = [".git", "node_modules", "target", "build", "dist", ".next", ".cache", "__pycache__", "venv"]

def find_command(directory: str, ignore_list: list[str]) -> str:
  cmd = ["find", directory]

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
