#! /usr/bin/env python3

import argparse
import trafilatura

def fetch(url: str) -> None:
    html = trafilatura.fetch_url(url=url)
    # Verificar se o HTML foi recuperado com sucesso
    if html:
        output = trafilatura.extract(filecontent=html, url=url)
        print(output)
    else:
        print('Failed to fetch HTML content from the provided URL.')

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fetch a web page using the url")
    parser.add_argument("--url", type=str, help="URL for search")
    args = parser.parse_args()

    fetch(args.url)
