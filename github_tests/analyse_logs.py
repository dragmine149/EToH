import json
import re
import sys

def append_to_all(list, log):
    for item in list:
        item.append(log)

def process_test_logs(log_entries):
    """
    Processes test logs to extract test suite and test results.

    Args:
        log_entries: A list of dictionaries, where each dictionary represents a log entry.

    Returns:
        A dictionary containing the results for each test suite.
    """
    test_suites = {}
    current_suite_logs = []

    for log in log_entries:
        text = log.get("text", "")
        # log_type = log.get("type", "")

        # Remove color arguments from the text for easier parsing
        text = re.sub(r"%c", "", text)
        # text = text
        text = re.sub(r"color:\s.*\s\s", "", text)

        append_to_all(current_suite_logs, log)

        if "(.gitignore)" in text: continue

        if "Starting test suite:" in text:
            suite_name_match = re.search(r"Starting test suite:\s*(.*?)$", text)
            if not suite_name_match: continue

            suite_name = suite_name_match.group(1).strip()
            test_suites[suite_name] = {
                "status": "Running",
                "passed": 0,
                "total": 0,
                "logs": [],
                "failed_tests": []
            }
            current_suite_logs.append([])
            continue

        if "Finished test suite:" in text:
            # print("-----------")
            # print(text)
            suite_result_match = re.search(r"Finished test suite:\s*(.*?)\s*(\w+)\s*\((\d+)/(\d+)\)!?$", text)
            # print(suite_result_match)
            if not suite_result_match: continue

            current = suite_result_match.group(1).strip()
            if current not in test_suites.keys(): continue

            status = suite_result_match.group(2)
            passed = int(suite_result_match.group(3))
            total = int(suite_result_match.group(4))

            test_suites[current]["status"] = status
            test_suites[current]["passed"] = passed
            test_suites[current]["total"] = total
            test_suites[current]["logs"] = current_suite_logs.pop()
            continue

    return test_suites

def print_github_output(test_suites):
    """
    Prints the test suite results in GitHub Actions workflow command format.

    Args:
        test_suites: A dictionary containing the results for each test suite.
    """
    failed = False
    print("::group::Test Suite Results")
    for suite_name, results in test_suites.items():
        print(f"Suite: {suite_name}")
        print(f"Status: {results['status']}")
        print(f"Result: {results['passed']}/{results['total']}")

        if results["status"] != "Passed":
            failed = True
            print("Failed Tests:")
            for failed_test in results["failed_tests"]:
                print(f"- {failed_test}")
            print("Logs:")
            for log in results["logs"]:
                 # Simple output for logs, you might want more structured output
                 # based on log type and content for better readability in GitHub Actions
                 print(f"  [{log.get('type', 'unknown').upper()}] {log.get('text', 'No text')}")

        print("-" * 30)
    print("::endgroup::")

    if failed:
        sys.exit("Some test failed!")


if __name__ == "__main__":
    logs = []
    with open("post_data.log") as f:
        lines = f.readlines()
        for line in lines:
            logs.append(json.loads(line))
    result = process_test_logs(logs)
    print_github_output(result)
