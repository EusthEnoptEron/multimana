import sys

# Set up logging
stdout_file = open('py_output.log', 'w', buffering = 1)
stderr_file = open('py_error.log', 'w', buffering = 1)
sys.stdout = stdout_file
sys.stderr = stderr_file

print("Set up logging")

import unreal_engine as ue
import datetime

def log(text, prefix):
    now = datetime.datetime.now().isoformat()
    print(f"{now} {prefix} python {text}")

def log_info(text):
    log(text, 'INFO')

def log_warn(text):
    log(text, 'WARN')

def log_error(text):
    log(text, 'ERROR')

ue.log = log_info
ue.log_warning = log_warn
ue.log_error = log_error
