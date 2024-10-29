import os
import stat

def is_dir(path):
    return stat.S_ISDIR(os.stat(path).st_mode)

for name in os.listdir('test-cases'):
    if is_dir('test-cases/' + name) and name[0:1] != '.' and name != 'target':
        os.system(f'python3 scripts/run-tests.py --detector={name}')
