#! /usr/bin/env python3

import argparse

from __common import file_exists, to_path

def write_file(path: str, content: str) -> None:
  file_path = to_path(path)
  file_path.parent.mkdir(parents=True, exist_ok=True)

  # Convert \n, \t, etc. into real characters
  content = content.encode("utf-8").decode("unicode_escape")

  with open(file_path, "w", encoding="utf-8") as f:
    f.write(content)


def main() -> None:
  parser = argparse.ArgumentParser(description="Write content to a file")
  parser.add_argument(
    "--path",
    type=str,
    help="File path to write"
  )
  parser.add_argument(
    "--content",
    type=str,
    help="Content to write into the file"
  )

  args = parser.parse_args()
  if not file_exists(args.path):
    print("File do not exists!")
    return

  write_file(args.path, args.content)
  print(f"Written to {args.path}")


if __name__ == "__main__":
  main()
