import utils
import json
import sys
import argparse
import re

def list_files_touched(base_ref, head_ref):
    code, out, err = utils.run_subprocess(['git', 'diff', '--raw', f'{base_ref}..{head_ref}'], ".")
    if code != 0:
        print('git diff failed: ' + err)
        exit(code)

    ret = []
    out = out if out != None else ''
    for line in out.split('\n'):
        if len(line) == 0:
            continue
        match = re.match(r'^\:\d{6} \d{6} [0-9a-fA-F]{8} [0-9a-fA-F]{8} [A-Z]\t(.+)$', line)
        if not match:
            print(f'Failed to parse diff line {line}')
            exit(-1)
        ret.append(match.group(1))

    return ret

def any_file_touched(files_touched, blockchain, detector):
    p1 = f'^test\\-cases\\/{re.escape(blockchain)}\\/{re.escape(detector)}\\/.*'
    p2 = f'^detectors\\/{re.escape(blockchain)}\\/{re.escape(detector)}\\/.*'
    for ft in files_touched:
        if re.match(p1, ft) or re.match(p2, ft):
            return True
    return False

def filter_test_cases(test_cases, files_touched):
    ret = []
    for test_case in test_cases:
        [blockchain, detector] = test_case.split('/')
        if any_file_touched(files_touched, blockchain, detector):
            ret.append(test_case)
    return ret


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
