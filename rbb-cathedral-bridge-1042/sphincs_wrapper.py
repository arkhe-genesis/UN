import ctypes
import os
import struct

# Caminho para a biblioteca
LIB_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "libsphincs.so"))
lib = ctypes.CDLL(LIB_PATH)

# Definir tipos das funções
lib.sphincs_keygen.argtypes = [
    ctypes.c_char_p,  # seed (16 bytes)
    ctypes.c_char_p,  # private_seed (16 bytes)
    ctypes.c_char_p   # public_root (16 bytes)
]
lib.sphincs_keygen.restype = None

lib.sphincs_sign.argtypes = [
    ctypes.c_char_p,  # message
    ctypes.c_size_t,  # msg_len
    ctypes.c_char_p,  # private_seed
    ctypes.c_char_p   # signature (output)
]
lib.sphincs_sign.restype = None

lib.sphincs_verify.argtypes = [
    ctypes.c_char_p,  # message
    ctypes.c_size_t,  # msg_len
    ctypes.c_char_p,  # signature
    ctypes.c_char_p   # public_root
]
lib.sphincs_verify.restype = ctypes.c_int

lib.sphincs_signature_size.argtypes = []
lib.sphincs_signature_size.restype = ctypes.c_size_t

SIG_SIZE = lib.sphincs_signature_size()

def sphincs_keygen(seed: bytes) -> tuple[bytes, bytes]:
    """Retorna (private_seed, public_root) ambos de 16 bytes."""
    if len(seed) != 16:
        raise ValueError("Seed deve ter 16 bytes")
    priv = ctypes.create_string_buffer(16)
    pub = ctypes.create_string_buffer(16)
    lib.sphincs_keygen(seed, priv, pub)
    return priv.raw, pub.raw

def sphincs_sign(message: bytes, private_seed: bytes) -> bytes:
    """Retorna assinatura de tamanho SIG_SIZE."""
    if len(private_seed) != 16:
        raise ValueError("private_seed deve ter 16 bytes")
    sig = ctypes.create_string_buffer(SIG_SIZE)
    lib.sphincs_sign(message, len(message), private_seed, sig)
    return sig.raw

def sphincs_verify(message: bytes, signature: bytes, public_root: bytes) -> bool:
    """Verifica assinatura; retorna True se válida."""
    if len(signature) != SIG_SIZE:
        raise ValueError(f"Assinatura deve ter {SIG_SIZE} bytes")
    if len(public_root) != 16:
        raise ValueError("public_root deve ter 16 bytes")
    return lib.sphincs_verify(message, len(message), signature, public_root) == 1
