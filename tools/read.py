#! /usr/bin/env python3

import argparse

from __common import file_exists, to_path

# Add extensions for non readable files for this tool
INVALID_EXTESIONS = ["mkv", "mp3", "mp4", "png", "jpg", "jpeg", "gif"]

def read_file(path: str) -> str:
  file_path = to_path(path)
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
  if not file_exists(args.path):
    print("File do not exists!")
    return

  ext = args.path.split(".")[-1]
  if ext in INVALID_EXTESIONS:
    print("Invalid file extension, can't be none of:", ", ".join(INVALID_EXTESIONS))
    return

  content = read_file(args.path)
  print(f"File content {content}")


if __name__ == "__main__":
  main()
