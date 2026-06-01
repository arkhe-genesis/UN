#include <stdint.h>
#include <stdlib.h>

// Negacyclic convolution C(x) = A(x) * B(x) mod (x^n + 1)
// C_k = sum_{i+j=k} A_i B_j - sum_{i+j=k+n} A_i B_j
void ntt_mul_c(int64_t* a, int64_t* b, int64_t* c, int n, int64_t q) {
    for (int i = 0; i < n; i++) {
        c[i] = 0;
    }
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            int idx = i + j;
            if (idx < n) {
                c[idx] = (c[idx] + a[i] * b[j]) % q;
            } else {
                c[idx - n] = (c[idx - n] - a[i] * b[j]) % q;
            }
        }
    }
    for (int i = 0; i < n; i++) {
        c[i] = (c[i] % q + q) % q;
    }
}
