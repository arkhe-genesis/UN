import os
import hashlib
import json
import time

class CathedralCoreutils:
    def __init__(self, theosis_threshold=0.3):
        self.theosis_threshold = theosis_threshold
        self.operations_log = []

    def mkdir(self, path, theosis_default=0.7):
        os.makedirs(path, exist_ok=True)
        self.theosis_set(path, theosis_default)
        return path

    def theosis_set(self, filepath, theosis):
        meta_path = f"{filepath}.theosis"
        with open(meta_path, 'w') as f:
            f.write(str(theosis))

    def get_file_theosis(self, filepath):
        meta_path = f"{filepath}.theosis"
        if os.path.exists(meta_path):
            with open(meta_path, 'r') as f:
                return float(f.read().strip())
        return 0.5

    def ls(self, directory, theosis_filter=None):
        result = []
        for filename in os.listdir(directory):
            filepath = os.path.join(directory, filename)
            if os.path.isfile(filepath) and not filename.endswith('.theosis'):
                theosis = self.get_file_theosis(filepath)
                if theosis_filter is None or theosis >= theosis_filter:
                    with open(filepath, 'rb') as f:
                        content = f.read()
                    hash_val = hashlib.sha3_256(content).hexdigest()
                    result.append({
                        'path': filepath,
                        'size': os.path.getsize(filepath),
                        'theosis': theosis,
                        'merkle_hash': hash_val[:32],
                        'seal': f"SEAL-{hash_val[:8]}",
                        'substrate_id': 'general'
                    })
        return sorted(result, key=lambda x: x['theosis'], reverse=True)
