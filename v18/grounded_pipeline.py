import hashlib
import numpy as np

class BiographicalMemory:
    @staticmethod
    def create_biographical_entry(state_vector: np.ndarray, metadata: dict) -> dict:
        return {
            "state_vector": state_vector.tolist() if isinstance(state_vector, np.ndarray) else state_vector,
            "metadata": metadata
        }

class UNBiographicalMemory(BiographicalMemory):
    @staticmethod
    def create_un_entry(state_vector: np.ndarray, metadata: dict,
                        agency: str, sensitivity: str = "unclassified") -> dict:
        entry = UNBiographicalMemory.create_biographical_entry(state_vector, metadata)
        entry["un_agency"] = agency
        entry["un_sensitivity"] = sensitivity
        entry["un_compliance_hash"] = hashlib.blake2b(
            (agency + sensitivity + str(metadata)).encode(), digest_size=16
        ).hexdigest()
        return entry
