# receive a name
# cd to test cases
# if the directory with that name exists
# go inside and create a new directory with
# name-n with n being how many directories there already are +1
# if the directory does not exist
# create a new directory with that name
# and inside create a directory with that name and -1

import os
import sys

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 new-test-case.py <test-case-name>")
        return

    test_case_name = sys.argv[1]
    test_case_path = os.path.join(os.getcwd(), "test-cases")
    test_case_dir = os.path.join(test_case_path, test_case_name)
    test_case_dir_n = os.path.join(test_case_dir, test_case_name + "-1")

    if os.path.exists(test_case_dir):
        n = len(os.listdir(test_case_dir))
        test_case_dir_n = os.path.join(test_case_dir, test_case_name + "-" + str(n + 1))
        os.mkdir(test_case_dir_n)
    else:
        os.mkdir(test_case_dir)
        os.mkdir(test_case_dir_n)

    os.mkdir(os.path.join(test_case_dir_n, "remediated-example"))
    os.mkdir(os.path.join(test_case_dir_n, "vulnerable-example"))

if __name__ == "__main__":
    main()
