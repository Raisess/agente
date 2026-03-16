#! /usr/bin/env python3

import argparse
from pathlib import Path

def read_file(path: str) -> str:
  file_path = Path(path)
  file_path.parent.mkdir(parents=True, exist_ok=True)

  # Convert \n, \t, etc. into real characters
  content = str()

  with open(file_path, "r", encoding="utf-8") as f:
      content = f.read()

  return content.strip()


def main():
  parser = argparse.ArgumentParser(description="Read a file content")
  parser.add_argument(
    "--path",
    type=str,
    help="File path to read"
  )

  args = parser.parse_args()
  content = read_file(args.path)
  print(f"File content {content}")


if __name__ == "__main__":
  main()
