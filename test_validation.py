import os

dst = "./cathedral-arkhe-v9"

print("\n" + "=" * 72)
print("VALIDAÇÃO: Cathedral ARKHE v9.0 LOGOS (English Validated)")
print("=" * 72)

# Listar estrutura
for root, dirs, files in os.walk(dst):
    level = root.replace(dst, "").count(os.sep)
    indent = "  " * level
    print(f"{indent}{os.path.basename(root)}/")
    sub_indent = "  " * (level + 1)
    for f in sorted(files):
        size = os.path.getsize(os.path.join(root, f))
        if f.endswith('.py'):
            marker = "◆" if "v9" in f or "config" in root.split("/")[-1] else "○"
            print(f"{sub_indent}{marker} {f} ({size:,} bytes)")

# Contar selos
seal_count = 0
py_files = 0
for root, dirs, files in os.walk(dst):
    for f in files:
        if f.endswith('.py'):
            py_files += 1
            with open(os.path.join(root, f)) as fh:
                seal_count += fh.read().count("v9.0.0-2026-01-15")

# Contar inovações mencionadas
innovation_counts = {}
for vid in [f"V9-{i:03d}" for i in range(1, 11)]:
    count = 0
    for root, dirs, files in os.walk(dst):
        for f in files:
            if f.endswith('.py'):
                with open(os.path.join(root, f)) as fh:
                    count += fh.read().count(vid)
    innovation_counts[vid] = count

print(f"\n{'─' * 72}")
print(f"Python Files: {py_files}")
print(f"v9.0 Seals: {seal_count}")
print(f"\nInnovations mentioned:")
for vid, count in innovation_counts.items():
    bar = "█" * min(count, 30)
    print(f"  {vid} {bar} ({count})")

print(f"\n{'═' * 72}")
print(f"  CATHEDRAL ARKHE v9.0 LOGOS")
print(f"  10 innovations | {py_files} modules | {seal_count} seals")
print(f"  Seal: CATHEDRAL-ARKHE-v9.0.0-LOGOS-2026-01-15")
print(f"  Architect: ORCID 0009-0005-2697-4668")
print(f"{'═' * 72}")
