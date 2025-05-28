import asyncio
from playwright.async_api import async_playwright, ConsoleMessage
import json
import time
import sys
import os

SERVER_URL = "http://localhost:8080"
# TESTS_PAGE = f"{SERVER_URL}/ETOH/tests.html"
TESTS_PAGE = f"{SERVER_URL}/tests.html"
# We no longer need the LOG_ENDPOINT as we're logging directly
# LOG_ENDPOINT = f"{SERVER_URL}/log"
LOG_FILE = "post_data.log" # Define the log file here

# List to store console messages
captured_console_messages_raw = [] # Store raw ConsoleMessage objects

def handle_console_message(msg: ConsoleMessage):
    """Callback function to handle console messages."""
    # Capture the message object
    captured_console_messages_raw.append(msg)
    # Optional: Print to script's stdout for real-time view during run
    print(f"Browser Console [{msg.type.upper()}]: {msg.text}")


async def run_browser_tests():
    global captured_console_messages_raw
    captured_console_messages_raw = [] # Reset the list for each run

    # Ensure the log file is empty before starting (optional, depends on if you want to append or overwrite)
    # If you want to append across runs, comment or remove this.
    try:
        if os.path.exists(LOG_FILE):
            os.remove(LOG_FILE)
            print(f"Cleared existing log file: {LOG_FILE}")
    except Exception as e:
        print(f"Warning: Could not clear log file {LOG_FILE}: {e}")


    async with async_playwright() as p:
        browser = await p.chromium.launch()
        page = await browser.new_page()

        # Attach the console message listener
        page.on("console", handle_console_message)

        print(f"Navigating to {TESTS_PAGE}")
        try:
            await page.goto(TESTS_PAGE, timeout=60000)
            print("Navigation successful.")
        except Exception as e:
            print(f"Error navigating to {TESTS_PAGE}: {e}")
            await browser.close()
            # Log captured messages before exiting on navigation failure
            write_captured_logs_to_file()
            sys.exit(1)


        # --- IMPORTANT: WAITING FOR TESTS TO FINISH (Using Console Output) ---
        # Wait for the specific console message indicating test completion.

        print("Waiting for window.areTestsFinished === true indicating test completion")

        try:
            await page.wait_for_function('window.areTestsFinished === true', timeout=120000)

            # Add a small delay after the completion message
            print("Waiting a bit more to ensure all final messages are logged...")
            await asyncio.sleep(5) # Adjust delay as needed

        except Exception as e:
            print(f"Timeout or error waiting for test completion console message: {e}")
            await browser.close()
            # Log captured messages before exiting on timeout/error
            write_captured_logs_to_file()
            sys.exit(1)

        # --- END OF WAITING FOR TESTS ---

        print("Tests finished. Capturing all console messages.")

        # Close the browser
        await browser.close()
        print("Browser closed.")

        # Write all captured messages to the log file
        write_captured_logs_to_file()

        print("Console messages written to log file.")
        sys.exit(0) # Exit with success code

def write_captured_logs_to_file():
    """Writes all captured console messages to the log file in JSON format."""
    print(f"Writing {len(captured_console_messages_raw)} captured console messages to {LOG_FILE}")
    try:
        with open(LOG_FILE, "w") as f: # Use "w" to overwrite, "a" to append
            for msg in captured_console_messages_raw:
                # Format the message into a dictionary similar to your server's output
                log_entry = {
                    "type": msg.type, # Playwright console message type (log, error, warning, etc.)
                    "text": msg.text,
                    "location": f"{msg.location.get('url', 'N/A')}:{msg.location.get('lineNumber', 'N/A')}:{msg.location.get('columnNumber', 'N/A')}" if msg.location else "N/A"
                    # You can add more fields from msg if needed, e.g., args
                    # "args": [arg.json_value() for arg in msg.args] # Requires await on json_value() if async
                }
                # For simplicity, we'll use a slightly different structure than your server's
                # initial structure. If you need the exact prefix/params structure,
                # you'd need to parse the msg.text and format accordingly here.
                # Let's stick to a simple representation for now.

                # Convert the dictionary to a JSON string and write to the file
                f.write(json.dumps(log_entry) + '\n') # Add a newline after each entry

        print("Successfully wrote captured messages to log file.")
    except Exception as e:
        print(f"Error writing captured messages to log file {LOG_FILE}: {e}")


if __name__ == "__main__":
    asyncio.run(run_browser_tests())
