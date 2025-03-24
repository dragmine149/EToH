import requests
import json
import time

def fetch_badges(cursor=None):
    base_url = "https://badges.roblox.com/v1/universes/3264581003/badges"
    params = {"limit": 100}
    if cursor:
        params["cursor"] = cursor

    response = requests.get(base_url, params=params)
    return response.json()

def main():
    all_badges = []
    next_cursor = None

    while True:
        result = fetch_badges(next_cursor)
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

    with open("badges.json", "w", encoding="utf-8") as f:
        json.dump(output_data, f, indent=2)

if __name__ == "__main__":
    main()
