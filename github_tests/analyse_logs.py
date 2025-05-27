import json
import sys
import os

LOG_FILE = "github_tests/post_data.log"
GITHUB_OUTPUT = os.getenv('GITHUB_OUTPUT') # Get the path to the GitHub output file

def log_to_github_output(name, value):
    """Logs a key-value pair to GitHub Actions output."""
    # Escape necessary characters for GitHub Actions output
    value = value.replace('%', '%25').replace('\n', '%0A').replace('\r', '%0D')
    with open(GITHUB_OUTPUT, 'a') as f:
        f.write(f'{name}={value}\n')

def analyze_log_file(log_file):
    test_results = {}
    current_test_logs = []
    in_test = False
    current_test_name = None

    try:
        with open(log_file, "r") as f:
            # Read the entire file content
            log_content = f.read()

        # Split the content into individual JSON objects
        # Assuming each JSON object is on a new line or can be split by a delimiter if you added one
        # If your server adds delimiters, adjust the split accordingly.
        # For now, we assume each log entry is a valid JSON object and we'll process line by line.
        # A more robust approach might involve parsing the whole file as a JSON array if your server writes it that way.
        # Given the server appends, we'll process line by line or by looking for JSON objects.
        # Let's try to parse potential JSON objects line by line, acknowledging this might need
        # adjustment based on the exact output format if it's not strictly one JSON per line.

        # A safer approach when appending JSON might be to treat the file as concatenated JSONs
        # and try to parse them sequentially. However, for simplicity and given the server appends,
        # reading line by line and attempting JSON parsing is a starting point.
        # Let's use a simple approach by splitting lines and trying to load JSON from each.
        # This might fail if a single log entry spans multiple lines without clear delimiters.
        # If your server logs are indeed one JSON object per line, this is fine.
        # If not, you'll need a more complex parser that can find and parse JSON objects
        # within the concatenated text.

        # Let's refine the parsing to handle potential multiple JSONs appended.
        # A simple split by '}{' and adding '{' and '}' back might work if they are directly concatenated.
        # But given the server appends, line-by-line is more likely intended if the server writes
        # with line breaks after each JSON. Let's assume each line is a potential JSON object.

        log_entries = log_content.strip().split('\n')
        parsed_entries = []
        for entry_str in log_entries:
            try:
                parsed_entries.append(json.loads(entry_str))
            except json.JSONDecodeError:
                # Handle lines that aren't valid JSON (e.g., server messages, if any)
                continue


        for entry in parsed_entries:
            current_test_logs.append(json.dumps(entry)) # Store the raw log entry

            if "prefix" in entry and isinstance(entry["prefix"], list):
                prefix_text = "".join(entry["prefix"]) # Join the prefix list

                # Check for the start of a test suite
                if "Starting test suite:" in prefix_text and "params" in entry and isinstance(entry["params"], list):
                    if in_test:
                        print(f"Warning: Found a new test start before the previous one ended.")
                        # Handle the previous test logs if needed, e.g., consider it failed or incomplete
                        # For now, we'll reset and start tracking the new one.
                        current_test_logs = [json.dumps(entry)] # Start new log tracking
                    in_test = True
                    current_test_name = entry["params"][0] if entry["params"] else "Unknown Test Suite"
                    test_results[current_test_name] = {"status": "running", "logs": current_test_logs}
                    current_test_logs = [] # Start a new list for logs within this test

                # Check for the end of a test suite
                elif "Finished test suite:" in prefix_text and "params" in entry and isinstance(entry["params"], list):
                    if not in_test:
                        print(f"Warning: Found a test end before a test start.")
                        continue # Ignore if we weren't tracking a test

                    result_params = entry["params"]
                    test_result = "Unknown"
                    if result_params:
                        test_result = result_params[0] # The result string (e.g., "Passed (4/4)!")

                    status = "Failed" if "Failed" in test_result or "Error" in test_result else "Passed"
                    test_results[current_test_name]["status"] = status
                    test_results[current_test_name]["result"] = test_result
                    test_results[current_test_name]["logs"].extend(current_test_logs) # Add remaining logs
                    in_test = False
                    current_test_name = None
                    current_test_logs = [] # Reset for the next potential test

                # Any other log entries within a test are added to current_test_logs
                elif in_test:
                   pass # Log is already added at the start of the loop

    except FileNotFoundError:
        print(f"Error: Log file not found at {log_file}")
        return {}
    except json.JSONDecodeError as e:
        print(f"Error decoding JSON from log file: {e}")
        return {}
    except Exception as e:
        print(f"An unexpected error occurred during log analysis: {e}")
        return {}

    # If a test started but didn't finish
    if in_test and current_test_name:
         print(f"Warning: Test suite '{current_test_name}' started but did not finish.")
         test_results[current_test_name]["status"] = "Incomplete"
         test_results[current_test_name]["logs"].extend(current_test_logs)


    return test_results

def format_log_entry_single_line(entry_json_string):
    """Formats a single JSON log entry into a single readable line."""
    try:
        entry = json.loads(entry_json_string)
        log_line = ""
        if "type" in entry:
            log_line += f"Type: {entry['type']} | "
        if "prefix" in entry and isinstance(entry["prefix"], list):
            # Filter out empty strings from prefix for cleaner output
            filtered_prefix = [p for p in entry["prefix"] if p]
            log_line += f"Prefix: {''.join(filtered_prefix)} | "
        if "params" in entry and isinstance(entry["params"], list):
            log_line += f"Params: {', '.join(map(str, entry['params']))}"

        # Remove trailing " | " if no params were added
        if log_line.endswith(" | "):
            log_line = log_line[:-3]

        return log_line.strip()
    except json.JSONDecodeError:
        return f"Invalid JSON entry: {entry_json_string.strip()}"
    except Exception as e:
        return f"Error formatting log entry: {entry_json_string.strip()} - {e}"


if __name__ == "__main__":
    results = analyze_log_file(LOG_FILE)

    all_tests_passed = True
    test_summary = []

    if not results:
        log_to_github_output("test_summary", "No test results found in log file.")
        sys.exit(1) # Indicate failure if no results are found

    for test_name, data in results.items():
        status = data.get("status", "Unknown")
        result_string = data.get("result", "N/A")
        test_summary.append(f"{test_name}: {status} ({result_string})")

        if status != "Passed":
            all_tests_passed = False
            log_to_github_output(f"{test_name.replace(' ', '_')}_status", status) # Log status
            log_to_github_output(f"{test_name.replace(' ', '_')}_result", result_string) # Log result

            # Log all log entries for failed/incomplete tests
            detailed_logs = "\n".join([format_log_entry_single_line(log_entry) for log_entry in data.get("logs", [])])
            log_to_github_output(f"{test_name.replace(' ', '_')}_logs", detailed_logs)

    # Log the overall test summary
    log_to_github_output("test_summary", "\n".join(test_summary))

    # Set the exit code based on whether all tests passed
    if all_tests_passed:
        print("All test suites passed!")
        sys.exit(0)
    else:
        print("Some test suites failed or were incomplete.")
        sys.exit(1) # Indicate failure
