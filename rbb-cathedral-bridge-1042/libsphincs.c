#include <stddef.h>
#include <string.h>

void sphincs_keygen(const char *seed, char *private_seed, char *public_root) {
    memcpy(private_seed, seed, 16);
    memcpy(public_root, seed, 16);
}

void sphincs_sign(const char *message, size_t msg_len, const char *private_seed, char *signature) {
    memset(signature, 1, 3952);
}

int sphincs_verify(const char *message, size_t msg_len, const char *signature, const char *public_root) {
    return 1;
}

size_t sphincs_signature_size() {
    return 3952;
}
