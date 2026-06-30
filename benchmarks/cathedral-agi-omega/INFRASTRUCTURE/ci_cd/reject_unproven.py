import sys
import subprocess

CRITICAL_DIRS = [
    "cathedral-agi-omega/ZK_REASONING_ENGINE/circuits",
    "cathedral-agi-omega/COGNITIVE_CORTEX/agents",
    "cathedral-agi-omega/DISTRIBUTED_COMPUTATION"
]

LEAN4_DIR = "cathedral-agi-omega/LEAN4_SUPEREGO"

def get_changed_files():
    try:
        # Get files changed between the target branch and the current PR branch
        # Assuming github actions provides the base branch in an env var or we just use origin/main
        result = subprocess.run(
            ["git", "diff", "--name-only", "origin/main...HEAD"],
            capture_output=True, text=True, check=True
        )
        return result.stdout.splitlines()
    except subprocess.CalledProcessError:
        print("Error getting git diff. Make sure you are in a git repository with origin/main fetched.")
        sys.exit(1)

def check_for_lean_proofs(changed_files):
    modifies_critical = False
    has_lean_proof = False

    for file in changed_files:
        if any(file.startswith(d) for d in CRITICAL_DIRS):
            modifies_critical = True
        if file.startswith(LEAN4_DIR) and file.endswith(".lean"):
            has_lean_proof = True

    if modifies_critical and not has_lean_proof:
        print("ERROR: CRITICAL VIOLATION.")
        print("You modified files in critical directories:")
        for file in changed_files:
            if any(file.startswith(d) for d in CRITICAL_DIRS):
                print(f" - {file}")
        print("\nHowever, no accompanying Lean 4 proof changes were found in LEAN4_SUPEREGO/.")
        print("Any change to reasoning engines, cognitive cortex, or distributed computation must be formally proven safe.")
        sys.exit(1)

    if modifies_critical and has_lean_proof:
        print("SUCCESS: Critical changes detected, and corresponding Lean 4 proofs are present.")
    else:
        print("SUCCESS: No critical directory changes requiring new proofs.")

if __name__ == "__main__":
    changed_files = get_changed_files()
    check_for_lean_proofs(changed_files)
