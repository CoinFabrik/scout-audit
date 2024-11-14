import utils
import json
import sys
import argparse


def print_json(test_cases):
    print(json.dumps(test_cases))


def print_list(test_cases):
    for tc in test_cases:
        print(tc)


def filter_by_blockchain(test_cases, blockchain):
    if not blockchain:
        return test_cases
    return [tc for tc in test_cases if tc.startswith(f"{blockchain}/")]


def main():
    parser = argparse.ArgumentParser(
        description="List test cases, optionally filtered by blockchain"
    )
    parser.add_argument(
        "--blockchain", "-b", help="Filter test cases by blockchain name"
    )
    parser.add_argument(
        "--format",
        "-f",
        choices=["json", "list"],
        default="json",
        help="Output format (default: json)",
    )
    args = parser.parse_args()

    test_cases = utils.list_test_cases()
    filtered_cases = filter_by_blockchain(test_cases, args.blockchain)

    if args.format == "json":
        print_json(filtered_cases)
    else:
        print_list(filtered_cases)


if __name__ == "__main__":
    main()
