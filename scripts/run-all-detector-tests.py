import test_utils
import utils

errors = []
for test_case in utils.list_test_cases():
    errors.extend(test_utils.run_tests(test_case))
test_utils.print_errors(errors)    
