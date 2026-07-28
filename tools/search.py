#! /usr/bin/env python3

import argparse
import os

from ddgs import DDGS

MAX_RESULTS = int(os.getenv("MAX_RESULTS", "5"))

def search(query: str) -> None:
  output = ""
  with DDGS() as ddgs:
    for r in ddgs.text(query, max_results=MAX_RESULTS, safe_search=False):
      output += "Title: " + r["title"] + "\n" + "Description: " + r["body"] + "\n" + "URL: " + r["href"] + "\n\n"

  print(output.strip())


if __name__ == "__main__":
  parser = argparse.ArgumentParser(description="Search the web with a query")
  parser.add_argument("--query", type=str, help="Query for the search engine")
  args = parser.parse_args()
  
  search(args.query)
