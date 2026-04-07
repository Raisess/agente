import os
from pathlib import Path

WORKING_DIR = os.getenv("WORKING_DIR", None)

def to_path(path: str) -> Path:
  path = Path(path)
  if WORKING_DIR not in str(path.absolute()):
    raise Exception(f"Invalid path, outside working dir: {WORKING_DIR}")

  return path
