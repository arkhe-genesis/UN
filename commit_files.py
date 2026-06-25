import os
import subprocess

# We need to make sure ALL files under cathedral-os are tracked by git.
subprocess.run(["git", "add", "cathedral-os/"], check=True)
subprocess.run(["git", "commit", "-m", "add cathedral-os root directory files", "--allow-empty"], check=True)
