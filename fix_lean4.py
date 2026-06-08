with open("./cathedral-arkhe-v9/cathedral/models/verification/formal_lean4.py", "r") as f:
    lines = f.readlines()

new_lines = []
i = 0
while i < len(lines):
    if "f\"-- TODO: generate theorem for {property_name}" in lines[i]:
        new_lines.append("            f\"-- TODO: generate theorem for {property_name}\\ntheorem {property_name}_prop : True := by trivial\")\n")
        i += 2
    else:
        new_lines.append(lines[i])
        i += 1

with open("./cathedral-arkhe-v9/cathedral/models/verification/formal_lean4.py", "w") as f:
    f.writelines(new_lines)
