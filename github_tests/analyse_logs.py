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
    test_suites = {} # Stores results per test suite
    current_suite_logs = [] # Stores all logs within the current test suite
    in_suite = False
    current_suite_name = None
    individual_test_statuses = {} # Track status of individual Expect Tests within the current suite

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

        for i, entry in enumerate(parsed_entries):
            log_entry_json_str = json.dumps(entry) # Get JSON string of current entry

            if "text" in entry and isinstance(entry["text"], str):
                log_text = entry["text"]

                # Check for the start of a new test suite
                start_match = re.search(r"%cStarting test suite:%c.*?(\s*(.*))$", log_text)
                if start_match:
                    suite_name = start_match.group(2).strip() if start_match.group(2) else "Unknown Test Suite"
                    print(f"Found 'Starting test suite:' entry: {suite_name}")

                    # If we were already in a suite, the previous one ended implicitly
                    if in_suite:
                        print(f"Warning: Found a new test suite start ('{suite_name}') before the previous one ('{current_suite_name}') explicitly ended.")
                        # Finalize the previous suite as incomplete
                        if current_suite_name and current_suite_name in test_suites:
                            test_suites[current_suite_name]["status"] = "Incomplete"
                            # The logs are already in current_suite_logs from the previous suite run
                        # The individual_test_statuses for the previous suite are already captured in test_suites entry.

                    # Start the new suite
                    in_suite = True
                    current_suite_name = suite_name
                    current_suite_logs = [log_entry_json_str] # Start new log tracking with the current entry
                    individual_test_statuses = {} # Reset individual test statuses for the new suite

                    test_suites[current_suite_name] = {
                        "status": "running", # Initially running
                        "result_summary": "N/A", # Will be updated by the end marker or implicit end
                        "logs": current_suite_logs, # Reference the current log list
                        "individual_test_statuses": individual_test_statuses # Reference individual statuses for the current suite
                    }

                # Check for the end of the current test suite
                end_match = re.search(r"%cFinished test suite:%c.*?(\s*(.*))$", log_text)
                # We are more specific now - check if the end marker text *contains* the current suite name
                # This handles cases where the end marker might appear out of order but matches a known running suite
                if in_suite and current_suite_name and current_suite_name in log_text and end_match:
                    end_suite_name_and_result = end_match.group(2).strip() if end_match.group(2) else "Unknown"
                    print(f"Found 'Finished test suite:' entry for current suite: {current_suite_name}")

                    suite_status = "Passed" # Assume passed unless individual tests failed or end marker indicates failure
                    final_result_summary = end_suite_name_and_result # Store the full text from the end marker

                    # Check individual test results collected for this suite
                    if any(status != "Passed" for status in individual_test_statuses.values()):
                         suite_status = "Failed"
                    # Also check the end marker text itself for "Failed" or "Error"
                    elif "Failed" in final_result_summary or "Error" in final_result_summary:
                         suite_status = "Failed"


                    test_suites[current_suite_name]["status"] = suite_status
                    test_suites[current_suite_name]["result_summary"] = final_result_summary
                    # Logs were already added as we went along
                    # Individual test statuses are already updated in individual_test_statuses which is referenced

                    print(f"Finished test suite: {current_suite_name} with final status: {suite_status}")

                    # End the current suite tracking
                    in_suite = False
                    current_suite_name = None
                    current_suite_logs = [] # Reset logs for the next suite
                    individual_test_statuses = {} # Reset individual statuses for the next suite

                # Look for the "Expect Test:" pattern (individual test results)
                # We only care about these if we are currently tracking a suite
                expect_test_match = re.search(r"%cExpect Test:%c\s*(.*?)\s*(Passed|Failed|Error)$", log_text)
                if in_suite and current_suite_name and expect_test_match:
                    individual_test_name = expect_test_match.group(1).strip()
                    result_status_text = expect_test_match.group(2).strip()

                    status = "Passed" if result_status_text == "Passed" else "Failed"

                    # Record the status of this individual test within the current suite's tracking
                    individual_test_statuses[individual_test_name] = status
                    print(f"Found individual test result within suite '{current_suite_name}': Test Name='{individual_test_name}', Status='{status}'")

                # If currently in a suite and this is not a start/end/expect marker,
                # add the log entry to the current suite's logs.
                # (This is already done at the beginning of the loop if in_suite is True)
                # else:
                #     if in_suite:
                #          pass # Log is already added
                #     else:
                #          # Log entry outside any tracked suite
                #          print(f"Ignoring log entry outside suite tracking: {log_text[:100]}...")
                #          pass # Optionally handle logs that occur before the first suite starts or after the last one ends


    except FileNotFoundError:
        print(f"Error: Log file not found at {log_file} (within try block)")
        return {}
    except json.JSONDecodeError as e:
        print(f"Critical Error decoding JSON from log file content: {e}")
        return {}
    except Exception as e:
        print(f"An unexpected error occurred during log analysis: {e}")
        return {}

    # If a test suite started but didn't finish by the end of the log file
    if in_suite and current_suite_name:
         print(f"Warning: Test suite '{current_suite_name}' started but did not finish at the end of the log file.")
         if current_suite_name in test_suites:
             test_suites[current_suite_name]["status"] = "Incomplete"
             # Logs and individual statuses are already captured in the test_suites entry.

    print(f"Finished analyzing log file. Found {len(test_suites)} test suites.")
    return test_suites

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
            # Simple formatting of the text content
            log_line += f"Text: {entry.get('text', 'N/A').strip()}"


        if log_line.endswith(" | "):
            log_line = log_line[:-3]

        return log_line.strip()
    except json.JSONDecodeError:
        return f"Invalid JSON entry: {entry_json_string.strip()}"
    except Exception as e:
        return f"Error formatting log entry: {entry_json_string.strip()} - {e}"


if __name__ == "__main__":
    print("Starting log analysis script.")
    suites = analyze_log_file(LOG_FILE) # Renamed 'results' to 'suites' for clarity

    all_suites_passed = True
    suite_summary_lines = []

    if not suites: # Check if any suites were found
        print("No test suites found in log file.")
        if os.path.exists(LOG_FILE) and os.path.getsize(LOG_FILE) > 0:
             log_to_github_output("test_summary", "Log file found, but no test suite start/end markers were identified.")
             try:
                 with open(LOG_FILE, "r") as f:
                      raw_content = f.read()
                      log_to_github_output("raw_log_content", raw_content)
             except Exception as e:
                 log_to_github_output("raw_log_content_error", f"Could not read raw log content: {e}")
        else:
             log_to_github_output("test_summary", "Log file not found or is empty.")

        sys.exit(1) # Indicate failure if no suites are found


    print(f"Processing {len(suites)} test suites for logging.")
    for suite_name, data in suites.items():
        suite_status = data.get("status", "Unknown")
        result_summary = data.get("result_summary", "N/A")
        summary_line = f"Suite '{suite_name}': {suite_status}"
        if result_summary != "N/A":
             summary_line += f" ({result_summary})"
        suite_summary_lines.append(summary_line)
        print(f"Processing suite: '{suite_name}', Status: {suite_status}, Result Summary: {result_summary}")

        # Determine if the overall suite should be marked as failed
        # This happens if the final status is not 'Passed' OR if any individual test within it failed.
        suite_failed = suite_status != "Passed" or any(status != "Passed" for status in data.get("individual_test_statuses", {}).values())


        if suite_failed:
            all_suites_passed = False
            print(f"Logging details for non-passed suite: '{suite_name}'")
            # Sanitize suite name for output keys
            safe_suite_name = re.sub(r'[^a-zA-Z0-9_]', '_', suite_name)
            log_to_github_output(f"{safe_suite_name}_status", suite_status)
            log_to_github_output(f"{safe_suite_name}_summary", result_summary)

            # Log the status of individual tests within this suite
            individual_results_summary = "Individual Test Results:\n"
            individual_statuses = data.get("individual_test_statuses", {})
            if individual_statuses:
                # Sort individual test results for consistent output
                sorted_individual_tests = sorted(individual_statuses.items())
                for test_name, status in sorted_individual_tests:
                    individual_results_summary += f"- {test_name}: {status}\n"
            else:
                individual_results_summary += "No individual test results (Expect Test:) logged for this suite."
            log_to_github_output(f"{safe_suite_name}_individual_results", individual_results_summary.strip())


            # Log all captured logs for this failing/incomplete suite
            detailed_logs = "\n".join([format_log_entry_single_line(log_entry) for log_entry in data.get("logs", [])])
            if detailed_logs:
                 log_to_github_output(f"{safe_suite_name}_logs", detailed_logs)
            else:
                 log_to_github_output(f"{safe_suite_name}_logs", "No detailed logs captured for this suite.")


    # Log the overall suite summary
    print("Logging overall suite summary.")
    log_to_github_output("suite_summary", "\n".join(suite_summary_lines))

    # Set the exit code based on whether all suites passed
    if all_suites_passed:
        print("All test suites passed! Exiting with code 0.")
        sys.exit(0)
    else:
        print("Some test suites failed or were incomplete. Exiting with code 1.")
        sys.exit(1)
