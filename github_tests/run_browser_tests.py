import asyncio
from playwright.async_api import async_playwright
import requests
import json
import time # Import time for potential delays

SERVER_URL = "http://localhost:8080"
TESTS_PAGE = f"{SERVER_URL}/tests"
LOG_ENDPOINT = f"{SERVER_URL}/log"

async def run_browser_tests():
    async with async_playwright() as p:
        browser = await p.chromium.launch() # Or .firefox, .webkit
        page = await browser.new_page()

        print(f"Navigating to {TESTS_PAGE}")
        try:
            await page.goto(TESTS_PAGE, timeout=60000) # 60 second timeout
            print("Navigation successful.")
        except Exception as e:
            print(f"Error navigating to {TESTS_PAGE}: {e}")
            await browser.close()
            return False # Indicate failure

        # --- IMPORTANT: WAITING FOR TESTS TO FINISH ---
        # This is the most critical part you need to adapt.
        # Here are a few common strategies:

        # Strategy 1: Wait for a specific element to appear on the page
        # indicating tests are done.
        # try:
        #     print("Waiting for test completion marker...")
        #     await page.wait_for_selector('#test-results-summary', timeout=120000) # Wait up to 2 minutes
        #     print("Test completion marker found.")
        # except Exception as e:
        #     print(f"Timeout waiting for test completion marker: {e}")
        #     await browser.close()
        #     return False

        # Strategy 2: Wait for a specific JavaScript variable or function
        # to be available or return a certain value.
        try:
            print("Waiting for JavaScript test completion flag...")
            await page.wait_for_function('window.areTestsFinished === true', timeout=120000)
            print("JavaScript test completion flag is true.")
        except Exception as e:
            print(f"Timeout waiting for JavaScript test completion flag: {e}")
            await browser.close()
            return False

        # Strategy 3: A fixed wait time (least reliable)
        # print("Waiting for a fixed duration for tests to run...")
        # time.sleep(30) # Wait for 30 seconds - ADJUST THIS AS NEEDED
        # print("Fixed wait complete.")

        # --- END OF WAITING FOR TESTS ---

        # After waiting, your web page's test suite should have run and
        # potentially logged results.

        # Send the required POST request to the server's log endpoint
        print(f"Sending POST request to {LOG_ENDPOINT}")
        log_data = {
            "type": 4,
            "prefix": [
                "%cTest results:%c",
                "color: orange",
                ""
            ],
        }
        try:
            response = requests.post(LOG_ENDPOINT, json=log_data)
            print(f"POST request status code: {response.status_code}")
            if response.status_code != 200:
                print(f"Error sending log data: {response.text}")
                await browser.close()
                return False # Indicate failure

        except requests.exceptions.RequestException as e:
            print(f"Error sending POST request to log endpoint: {e}")
            await browser.close()
            return False # Indicate failure


        print("Browser tests run and log request sent.")
        await browser.close()
        return True # Indicate success

if __name__ == "__main__":
    if asyncio.run(run_browser_tests()):
        print("Browser test script finished successfully.")
        exit(0) # Exit with success code
    else:
        print("Browser test script failed.")
        exit(1) # Exit with failure code
