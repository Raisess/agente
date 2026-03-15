#! /usr/bin/env python3

import argparse
from pathlib import Path

def write_file(path, content):
    file_path = Path(path)
    file_path.parent.mkdir(parents=True, exist_ok=True)

    with open(file_path, "w", encoding="utf-8") as f:
        f.write(content)


def main():
    parser = argparse.ArgumentParser(description="Write content to a file")
    parser.add_argument(
        "path",
        help="File path to write"
    )
    parser.add_argument(
        "content",
        help="Content to write into the file"
    )

    args = parser.parse_args()
    write_file(args.path, args.content)
    print(f"Written to {args.path}")


if __name__ == "__main__":
    main()
