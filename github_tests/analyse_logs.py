import json
import sys
import os
import re # Import regex module

LOG_FILE = "post_data.log"
GITHUB_OUTPUT = os.getenv('GITHUB_OUTPUT')

def log_to_github_output(name, value):
    """Logs a key-value pair to GitHub Actions output."""
    value = value.replace('%', '%25').replace('\n', '%0A').replace('\r', '%0D')
    try:
        with open(GITHUB_OUTPUT, 'a') as f:
            f.write(f'{name}={value}\n')
    except Exception as e:
        print(f"Error writing to GitHub output file {GITHUB_OUTPUT}: {e}")


def analyze_log_file(log_file):
    print(f"Analyzing log file: {log_file}")
    test_results = {}
    current_test_logs = []
    # We will now track tests individually based on the "Expect Test:" pattern
    # We no longer need a global 'in_test' state for a full suite,
    # but rather process each "Expect Test:" log entry as a test result.
    # We will group logs per test if you have a way to identify them.
    # For now, let's process each "Expect Test:" line as a result entry.
    # If there are other logs related to a specific test that you want to group,
    # we'll need a way to associate them (e.g., based on time, or if your tests
    # log a start/end boundary for each individual "Expect Test:").

    if not os.path.exists(log_file):
        print(f"Error: Log file not found at {log_file}")
        return {}

    try:
        with open(log_file, "r") as f:
            log_content = f.read()
        print(f"Successfully read log file. Content length: {len(log_content)}")

        log_entries_raw = log_content.strip().split('\n')
        parsed_entries = []
        for i, entry_str in enumerate(log_entries_raw):
            if not entry_str.strip():
                continue
            try:
                parsed_entries.append(json.loads(entry_str))
            except json.JSONDecodeError as e:
                print(f"Error decoding JSON from line {i}: {entry_str[:100]}... Error: {e}")
            except Exception as e:
                 print(f"Unexpected error parsing line {i}: {entry_str[:100]}... Error: {e}")

        print(f"Successfully parsed {len(parsed_entries)} valid JSON entries.")

        # We'll iterate through parsed entries and look for test results
        for i, entry in enumerate(parsed_entries):
            # Store all parsed entries in current_test_logs for now,
            # as we might want to show all logs if any test fails.
            # A more sophisticated approach would group logs by test if possible.
            current_test_logs.append(json.dumps(entry))

            if "text" in entry and isinstance(entry["text"], str):
                log_text = entry["text"]

                # Look for the "Expect Test:" pattern
                match = re.search(r"%cExpect Test:%c\s*(.*?)\s*(Passed|Failed|Error)$", log_text)

                if match:
                    # Found a test result entry
                    test_name = match.group(1).strip()
                    result_status_text = match.group(2).strip() # Passed, Failed, or Error

                    status = "Passed" if result_status_text == "Passed" else "Failed" # Assuming Failed/Error means failure

                    # Store the result for this specific test name
                    # If a test name appears multiple times, the last one wins.
                    # You might need to refine this if you have multiple checks
                    # within a single named "Expect Test".
                    test_results[test_name] = {
                        "status": status,
                        "result": result_status_text,
                        # We are not currently grouping logs per test in this version.
                        # If you need that, you'll need a way to identify logs
                        # belonging to a specific test run (e.g., timestamps, test ID in log).
                        "logs": [json.dumps(entry)] # For simplicity, only include the result line itself for now
                    }
                    print(f"Found test result: Test Name='{test_name}', Status='{status}', Result='{result_status_text}'")

                # If you had other logs you wanted to associate with tests,
                # you would add logic here to group them based on surrounding
                # "Expect Test:" entries or other markers.

    except FileNotFoundError:
        print(f"Error: Log file not found at {log_file} (within try block)")
        return {}
    except json.JSONDecodeError as e:
        print(f"Critical Error decoding JSON from log file content: {e}")
        return {}
    except Exception as e:
        print(f"An unexpected error occurred during log analysis: {e}")
        return {}

    # In this version, we process individual test results as they appear.
    # We don't have a concept of an overall test suite start/end anymore,
    # so we don't check for incomplete suites in the same way.

    print(f"Finished analyzing log file. Found {len(test_results)} individual test results.")
    return test_results

def format_log_entry_single_line(entry_json_string):
    """Formats a single JSON log entry into a single readable line from the new format."""
    try:
        entry = json.loads(entry_json_string)
        log_line = ""
        if "type" in entry:
            log_line += f"Type: {entry.get('type', 'N/A').upper()} | "
        if "location" in entry:
             log_line += f"Location: {entry.get('location', 'N/A')} | "
        if "text" in entry:
            log_line += f"Text: {entry.get('text', 'N/A')}"


        if log_line.endswith(" | "):
            log_line = log_line[:-3]

        return log_line.strip()
    except json.JSONDecodeError:
        return f"Invalid JSON entry: {entry_json_string.strip()}"
    except Exception as e:
        return f"Error formatting log entry: {entry_json_string.strip()} - {e}"


if __name__ == "__main__":
    print("Starting log analysis script.")
    results = analyze_log_file(LOG_FILE)

    all_tests_passed = True
    test_summary = []

    if not results:
        print("No individual test results found in log file.")
        # Check if the log file exists but contains no test results
        if os.path.exists(LOG_FILE) and os.path.getsize(LOG_FILE) > 0:
             # If file exists and has content but no test results found, might indicate parsing issue or no tests ran
             log_to_github_output("test_summary", "Log file found, but no individual test results (Expect Test:) were identified.")
             # Optionally log the raw content for debugging
             try:
                 with open(LOG_FILE, "r") as f:
                      raw_content = f.read()
                      log_to_github_output("raw_log_content", raw_content)
             except Exception as e:
                 log_to_github_output("raw_log_content_error", f"Could not read raw log content: {e}")
        else:
             log_to_github_output("test_summary", "Log file not found or is empty.")

        sys.exit(1) # Indicate failure if no results are found


    print(f"Processing {len(results)} individual test results for logging.")
    for test_name, data in results.items():
        status = data.get("status", "Unknown")
        result_string = data.get("result", "N/A")
        summary_line = f"{test_name}: {status}"
        if result_string != "N/A":
             summary_line += f" ({result_string})"
        test_summary.append(summary_line)
        print(f"Processing test result: {test_name}, Status: {status}, Result: {result_string}")


        if status != "Passed":
            all_tests_passed = False
            print(f"Logging details for non-passed test result: {test_name}")
            # Sanitize test name for output keys
            safe_test_name = re.sub(r'[^a-zA-Z0-9_]', '_', test_name)
            log_to_github_output(f"{safe_test_name}_status", status)
            log_to_github_output(f"{safe_test_name}_result", result_string)

            # In this version, 'logs' only contains the result line itself.
            # If you need all logs related to a test, you would need to
            # group them in the analyze_log_file function.
            # For now, we'll just log the result line as the detailed log.
            detailed_logs = "\n".join([format_log_entry_single_line(log_entry) for log_entry in data.get("logs", [])])

            # If you wanted to log *all* console output captured during the run
            # when any test fails, you would iterate through the 'current_test_logs'
            # list that was populated *before* the test results were processed
            # and log that. Be mindful of potentially large log output.
            # For now, we stick to the log entry specifically identified as a result.

            if detailed_logs:
                 log_to_github_output(f"{safe_test_name}_logs", detailed_logs)
            else:
                 # This case shouldn't happen if 'logs' always contains the result line
                 log_to_github_output(f"{safe_test_name}_logs", "No detailed log entry available for this result.")


    # Log the overall test summary
    print("Logging overall test summary.")
    log_to_github_output("test_summary", "\n".join(test_summary))

    # Set the exit code based on whether all tests passed
    if all_tests_passed:
        print("All test results passed! Exiting with code 0.")
        sys.exit(0)
    else:
        print("Some test results failed or had errors. Exiting with code 1.")
        sys.exit(1)
