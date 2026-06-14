#!/usr/bin/env python3
import os
import sys

def run_checkpoint():
    print("=== CHECKPOINT FASE 1 ===")
    print()
    print("[1] OS Info:")
    os.system("uname -a")
    os.system("cat /etc/os-release | grep PRETTY_NAME")
    print()
    print("[2] NVIDIA Driver:")
    os.system("nvidia-smi --query-gpu=driver_version,name --format=csv,noheader || echo 'N/A'")
    print()
    print("[3] CUDA:")
    os.system("nvcc --version | grep release || echo 'N/A'")
    print()
    print("[4] Python:")
    print(sys.version)
    print()
    print("[5] PyTorch + CUDA:")
    try:
        import torch
        print(f"  CUDA: {torch.cuda.is_available()}, GPUs: {torch.cuda.device_count()}, Device 0: {torch.cuda.get_device_name(0) if torch.cuda.device_count() > 0 else 'N/A'}")
    except ImportError:
        print("  ❌ PyTorch NAO ENCONTRADO")
    print()
    print("[6] Rust:")
    os.system("rustc --version || echo 'rustc NAO ENCONTRADO'")
    os.system("cargo --version || echo 'cargo NAO ENCONTRADO'")
    print()
    print("[8] Diretórios:")
    os.system("ls -la .")
    print()
    print("[9] Dependências Python críticas:")
    mods = ['torch','numpy','aiohttp','transformers','z3','cv2','ultralytics','timm','hnswlib','yaml','prometheus_client']
    for m in mods:
        try:
            __import__(m)
            print(f"  ✅ {m}")
        except ImportError:
            print(f"  ❌ {m}")
    print()
    print("=== FIM CHECKPOINT ===")

if __name__ == "__main__":
    run_checkpoint()
