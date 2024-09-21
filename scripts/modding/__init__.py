import sys

# Set up logging
stdout_file = open('py_output.log', 'w', buffering = 1)
stderr_file = open('py_error.log', 'w', buffering = 1)
sys.stdout = stdout_file
sys.stderr = stderr_file

print("Set up logging")

import unreal_engine as ue
import mod_extensions

# We're redirecting the ue log functions to our internal logging handler
def log(text, severity):
    mod_extensions.log(text, severity)

def log_info(text):
    log(text, 2)

def log_warn(text):
    log(text, 1)

def log_error(text):
    log(text, 0)

ue.log = log_info
ue.log_warning = log_warn
ue.log_error = log_error
