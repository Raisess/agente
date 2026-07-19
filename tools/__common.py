import os
from pathlib import Path

IGNORE_WORKING_DIR = bool(os.getenv("IGNORE_WORKING_DIR", 0))
WORKING_DIR = os.getenv("WORKING_DIR", None)

def to_path(path: str) -> Path:
  path = Path(path)
  if IGNORE_WORKING_DIR == False and WORKING_DIR not in str(path.absolute()):
    raise Exception(f"Invalid path, outside working dir: {WORKING_DIR}")

  return path


def file_exists(path: str) -> bool:
  return os.path.exists(path)
