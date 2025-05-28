import json
import sys
import os

LOG_FILE = "post_data.log"
GITHUB_OUTPUT = os.getenv('GITHUB_OUTPUT') # Get the path to the GitHub output file

def log_to_github_output(name, value):
    """Logs a key-value pair to GitHub Actions output."""
    # Escape necessary characters for GitHub Actions output
    # print(f"Attempting to log to GitHub output: {name}={value}") # Debugging print
    value = value.replace('%', '%25').replace('\n', '%0A').replace('\r', '%0D')
    try:
        with open(GITHUB_OUTPUT, 'a') as f:
            f.write(f'{name}={value}\n')
        # print(f"Successfully logged {name} to GitHub output.") # Debugging print
    except Exception as e:
        print(f"Error writing to GitHub output file {GITHUB_OUTPUT}: {e}") # Debugging print


def analyze_log_file(log_file):
    print(f"Analyzing log file: {log_file}") # Debugging print
    test_results = {}
    current_test_logs = []
    in_test = False
    current_test_name = None

    if not os.path.exists(log_file):
        print(f"Error: Log file not found at {log_file}") # Debugging print
        return {} # Returning empty dictionary, will lead to exit(1) later

    try:
        with open(log_file, "r") as f:
            log_content = f.read()
        print(f"Successfully read log file. Content length: {len(log_content)}") # Debugging print
        # print(f"Log file content:\n---\n{log_content}\n---") # Debugging: Print the whole file content

        log_entries = log_content.strip().split('\n')
        print(f"Split log content into {len(log_entries)} potential lines.") # Debugging print
        parsed_entries = []
        for i, entry_str in enumerate(log_entries):
            if not entry_str.strip(): # Skip empty lines
                continue
            # print(f"Attempting to parse line {i}: {entry_str[:100]}...") # Debugging print
            try:
                parsed_entries.append(json.loads(entry_str))
                # print(f"Successfully parsed line {i}.") # Debugging print
            except json.JSONDecodeError as e:
                print(f"Error decoding JSON from line {i}: {entry_str[:100]}... Error: {e}") # Debugging print
                # Continue processing other lines
            except Exception as e:
                 print(f"Unexpected error parsing line {i}: {entry_str[:100]}... Error: {e}") # Debugging print


        print(f"Successfully parsed {len(parsed_entries)} valid JSON entries.") # Debugging print

        for i, entry in enumerate(parsed_entries):
            # print(f"Processing entry {i}: {entry}") # Debugging print
            current_test_logs.append(json.dumps(entry)) # Store the raw log entry

            if "prefix" in entry and isinstance(entry["prefix"], list):
                prefix_text = "".join(entry["prefix"])

                if "Starting test suite:" in prefix_text and "params" in entry and isinstance(entry["params"], list):
                    print(f"Found 'Starting test suite:' entry.") # Debugging print
                    if in_test:
                        print(f"Warning: Found a new test start before the previous one ended. Test: {current_test_name}") # Debugging print
                        # Handle the previous test logs if needed
                        current_test_logs = [json.dumps(entry)] # Start new log tracking
                    in_test = True
                    current_test_name = entry["params"][0] if entry["params"] else "Unknown Test Suite"
                    print(f"Starting test: {current_test_name}") # Debugging print
                    test_results[current_test_name] = {"status": "running", "logs": current_test_logs}
                    current_test_logs = []

                elif "Finished test suite:" in prefix_text and "params" in entry and isinstance(entry["params"], list):
                    print(f"Found 'Finished test suite:' entry.") # Debugging print
                    if not in_test:
                        print(f"Warning: Found a test end before a test start.") # Debugging print
                        continue

                    result_params = entry["params"]
                    test_result = "Unknown"
                    if result_params:
                        test_result = result_params[0]

                    status = "Failed" if "Failed" in test_result or "Error" in test_result else "Passed"
                    print(f"Finishing test: {current_test_name} with status {status}") # Debugging print
                    test_results[current_test_name]["status"] = status
                    test_results[current_test_name]["result"] = test_result
                    test_results[current_test_name]["logs"].extend(current_test_logs)
                    in_test = False
                    current_test_name = None
                    current_test_logs = []

                elif in_test:
                   # print(f"Adding log entry to current test logs: {entry_str[:50]}...") # Debugging print
                   pass # Log is already added at the start of the loop

    except FileNotFoundError:
        # This case is now handled by the initial check, but keeping the print
        print(f"Error: Log file not found at {log_file} (within try block)") # Debugging print
        return {}
    except json.JSONDecodeError as e:
        print(f"Critical Error decoding JSON from log file content: {e}") # Debugging print
        return {}
    except Exception as e:
        print(f"An unexpected error occurred during log analysis: {e}") # Debugging print
        return {}

    # If a test started but didn't finish
    if in_test and current_test_name:
         print(f"Warning: Test suite '{current_test_name}' started but did not finish.") # Debugging print
         if current_test_name in test_results:
             test_results[current_test_name]["status"] = "Incomplete"
             test_results[current_test_name]["logs"].extend(current_test_logs)
         else:
             # Handle case where start was found but dictionary entry wasn't created (less likely)
             print(f"Error: Test '{current_test_name}' was in_test but no entry in test_results.") # Debugging print
             test_results[current_test_name] = {"status": "Incomplete", "logs": current_test_logs}


    print(f"Finished analyzing log file. Found {len(test_results)} test suites.") # Debugging print
    return test_results

def format_log_entry_single_line(entry_json_string):
    """Formats a single JSON log entry into a single readable line."""
    try:
        entry = json.loads(entry_json_string)
        log_line = ""
        if "type" in entry:
            log_line += f"Type: {entry['type']} | "
        if "prefix" in entry and isinstance(entry["prefix"], list):
            filtered_prefix = [p for p in entry["prefix"] if p]
            log_line += f"Prefix: {''.join(filtered_prefix)} | "
        if "params" in entry and isinstance(entry["params"], list):
            log_line += f"Params: {', '.join(map(str, entry['params']))}"

        if log_line.endswith(" | "):
            log_line = log_line[:-3]

        return log_line.strip()
    except json.JSONDecodeError:
        return f"Invalid JSON entry: {entry_json_string.strip()}"
    except Exception as e:
        return f"Error formatting log entry: {entry_json_string.strip()} - {e}"


if __name__ == "__main__":
    print("Starting log analysis script.") # Debugging print
    results = analyze_log_file(LOG_FILE)

    all_tests_passed = True
    test_summary = []

    if not results:
        print("No test results found in log file. Logging failure.") # Debugging print
        log_to_github_output("test_summary", "No test results found in log file.")
        sys.exit(1) # Indicate failure if no results are found

    print("Processing test results for logging.") # Debugging print
    for test_name, data in results.items():
        status = data.get("status", "Unknown")
        result_string = data.get("result", "N/A")
        summary_line = f"{test_name}: {status}"
        if status in ["Passed", "Failed", "Incomplete"]: # Add result string for finished tests
             summary_line += f" ({result_string})"
        test_summary.append(summary_line)
        print(f"Processing test: {test_name}, Status: {status}, Result: {result_string}") # Debugging print


        if status != "Passed":
            all_tests_passed = False
            print(f"Logging details for non-passed test: {test_name}") # Debugging print
            log_to_github_output(f"{test_name.replace(' ', '_')}_status", status)
            log_to_github_output(f"{test_name.replace(' ', '_')}_result", result_string)

            detailed_logs = "\n".join([format_log_entry_single_line(log_entry) for log_entry in data.get("logs", [])])
            if detailed_logs:
                 log_to_github_output(f"{test_name.replace(' ', '_')}_logs", detailed_logs)
            else:
                 log_to_github_output(f"{test_name.replace(' ', '_')}_logs", "No detailed logs available for this test.")


    # Log the overall test summary
    print("Logging overall test summary.") # Debugging print
    log_to_github_output("test_summary", "\n".join(test_summary))

    # Set the exit code based on whether all tests passed
    if all_tests_passed:
        print("All test suites passed! Exiting with code 0.") # Debugging print
        sys.exit(0)
    else:
        print("Some test suites failed or were incomplete. Exiting with code 1.") # Debugging print
        sys.exit(1) # Indicate failure
