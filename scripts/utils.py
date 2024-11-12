import subprocess
import stat
import os

RED = "\033[91m"
GREEN = "\033[92m"
ENDC = "\033[0m"

def run_subprocess(command: list, cwd: str):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    stdout = result.stdout.strip() if result.stdout else None
    stderr = result.stderr.strip() if result.stderr else None
    return (result.returncode, stdout, stderr)

def get_or_default(map, k, default):
    return map[k] if k in map else default

def simple_runner(name, commands, opts):
    show_err = get_or_default(opts, "show_err", True)
    fail_fast = get_or_default(opts, "fail_fast", True)
    ret = 0
    for (wd, cmd) in commands:
        retcode, out, err = run_subprocess(cmd.split(' '), wd)
        if retcode != 0:
            if show_err:
                print(f"{RED}Error running {name} on {wd}:")
                print(f"{err}{ENDC}")
            else:
                print(f"{RED}Error running {name} on {wd}{ENDC}")
            if fail_fast:
                return retcode
            ret = retcode
    return ret

def list_test_cases():
    ret = []
    for blockchain in os.listdir('detectors'):
        path = f'detectors/{blockchain}/test-cases'
        if not os.path.isdir(path):
            continue
        for test_case in os.listdir(path):
            if not os.path.isdir(f'{path}/{test_case}'):
                continue
            if test_case == 'target':
                continue
            if test_case[0:1] == '.':
                continue
            ret.append(f'{blockchain}/{test_case}')
    return ret
