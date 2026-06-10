import os
import sys
import subprocess

def get_changed_files():
    """Get the list of changed files in the latest commit or PR."""
    try:
        # For simplicity, assuming this runs on PRs comparing against main.
        # In a real GitHub action: `git diff --name-only origin/main...HEAD`
        result = subprocess.run(
            ['git', 'diff', '--name-only', 'HEAD~1', 'HEAD'],
            capture_output=True, text=True, check=True
        )
        return result.stdout.strip().split('\n')
    except subprocess.CalledProcessError:
        print("Error getting changed files from git.")
        return []

def main():
    critical_paths = [
        "ZK_REASONING_ENGINE/circuits",
        "COGNITIVE_CORTEX/agents",
        "DISTRIBUTED_COMPUTATION"
    ]

    changed_files = get_changed_files()

    touches_critical = False
    lean_proof_included = False

    for file in changed_files:
        if not file: continue

        # Check if it touches a critical path
        for path in critical_paths:
            if path in file:
                touches_critical = True
                print(f"Detected change in critical path: {file}")

        # Check if a lean proof is included
        if file.endswith(".lean"):
            lean_proof_included = True
            print(f"Detected Lean proof: {file}")

    if touches_critical and not lean_proof_included:
        print("\nERROR: Changes to critical directories require an accompanying Lean 4 proof update (.lean file).")
        print("Critical directories touched but no .lean file was found in the commit.")
        sys.exit(1)

    print("CI/CD Check Passed: CathedralAGI constraints satisfied.")
    sys.exit(0)

if __name__ == "__main__":
    main()
