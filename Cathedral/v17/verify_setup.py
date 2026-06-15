import sys
import logging

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(name)s] %(levelname)s: %(message)s")

def check_dependencies():
    deps = [
        "torch", "numpy", "aiohttp", "transformers", "z3",
        "cv2", "ultralytics", "timm", "hnswlib", "yaml", "prometheus_client"
    ]
    missing = []
    for dep in deps:
        try:
            __import__(dep)
            logging.info(f"✅ {dep} imported successfully")
        except ImportError:
            logging.error(f"❌ {dep} failed to import")
            missing.append(dep)

    if missing:
        logging.error(f"Missing dependencies: {', '.join(missing)}")
        sys.exit(1)
    else:
        logging.info("All dependencies verified successfully.")

if __name__ == "__main__":
    check_dependencies()
