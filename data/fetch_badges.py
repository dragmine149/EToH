import requests
import json
import time
import argparse

def fetch_badges(use_old_url=False, cursor=None):
    old_url = "https://badges.roblox.com/v1/universes/1055653882/badges"
    base_url = "https://badges.roblox.com/v1/universes/3264581003/badges"
    params = {"limit": 100}
    if cursor:
        params["cursor"] = cursor

    url = old_url if use_old_url else base_url
    response = requests.get(url, params=params)
    return response.json()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--old', action='store_true', help='Use old URL')
    args = parser.parse_args()

    all_badges = []
    next_cursor = None

    while True:
        result = fetch_badges(args.old, next_cursor)
        all_badges.extend(result["data"])

        next_cursor = result.get("nextPageCursor")
        if not next_cursor:
            break

        # Add a small delay to avoid rate limiting
        time.sleep(0.5)

    output_data = {
        "previousPageCursor": None,
        "nextPageCursor": next_cursor,
        "data": all_badges
    }

    filename = "old_badges.json" if args.old else "badges.json"
    with open(filename, "w", encoding="utf-8") as f:
        json.dump(output_data, f, indent=2)

if __name__ == "__main__":
    main()
