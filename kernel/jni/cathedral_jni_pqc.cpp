//! Cathedral ARKHE — JNI Bindings para PQC (ML-DSA, SLH-DSA, ML-KEM)
//! Selo: CATHEDRAL-ARKHE-JNI-PQC-v1.0.0-2026-06-21

#include <jni.h>
#include <string>
#include <vector>
#include <cstring>
#include "cathedral_pqc.h"

// ============================================================================
// Exceção Helper
// ============================================================================

inline void throwCathedralException(JNIEnv* env, const char* message) {
    jclass clazz = env->FindClass("cathedral/CathedralException");
    if (clazz != nullptr) {
        env->ThrowNew(clazz, message);
    }
}

// ============================================================================
// ML-DSA (Dilithium) — FIPS 204
// ============================================================================

extern "C" {

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_MLDSA_nativeGenerateKeypair(
    JNIEnv* env,
    jclass clazz,
    jint level
) {
    CathedralMldsaKeyPair kp = cathedral_mldsa_generate_keypair(level);
    if (kp.public_key == nullptr || kp.private_key == nullptr) {
        throwCathedralException(env, "Falha ao gerar par de chaves ML-DSA");
        return nullptr;
    }

    // Serializa: [public_key_len, public_key, private_key_len, private_key]
    size_t total_len = sizeof(uint32_t) + kp.public_key_len + sizeof(uint32_t) + kp.private_key_len;
    std::vector<uint8_t> buffer(total_len);
    size_t offset = 0;

    uint32_t pub_len = kp.public_key_len;
    memcpy(buffer.data() + offset, &pub_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, kp.public_key, pub_len);
    offset += pub_len;

    uint32_t priv_len = kp.private_key_len;
    memcpy(buffer.data() + offset, &priv_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, kp.private_key, priv_len);

    cathedral_free(kp.public_key);
    cathedral_free(kp.private_key);

    jbyteArray result = env->NewByteArray(static_cast<jsize>(total_len));
    env->SetByteArrayRegion(result, 0, total_len, reinterpret_cast<const jbyte*>(buffer.data()));
    return result;
}

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_MLDSA_nativeSign(
    JNIEnv* env,
    jclass clazz,
    jbyteArray message,
    jbyteArray privateKey
) {
    jsize msg_len = env->GetArrayLength(message);
    jbyte* msg = env->GetByteArrayElements(message, nullptr);
    jsize priv_len = env->GetArrayLength(privateKey);
    jbyte* priv = env->GetByteArrayElements(privateKey, nullptr);

    size_t sig_len;
    uint8_t* signature = cathedral_mldsa_sign(
        reinterpret_cast<uint8_t*>(msg), msg_len,
        reinterpret_cast<uint8_t*>(priv), priv_len,
        &sig_len
    );

    env->ReleaseByteArrayElements(message, msg, JNI_ABORT);
    env->ReleaseByteArrayElements(privateKey, priv, JNI_ABORT);

    if (signature == nullptr) {
        throwCathedralException(env, "Falha na assinatura ML-DSA");
        return nullptr;
    }

    jbyteArray result = env->NewByteArray(static_cast<jsize>(sig_len));
    env->SetByteArrayRegion(result, 0, sig_len, reinterpret_cast<const jbyte*>(signature));
    cathedral_free(signature);
    return result;
}

JNIEXPORT jboolean JNICALL
Java_cathedral_pqc_MLDSA_nativeVerify(
    JNIEnv* env,
    jclass clazz,
    jbyteArray message,
    jbyteArray signature,
    jbyteArray publicKey
) {
    jsize msg_len = env->GetArrayLength(message);
    jbyte* msg = env->GetByteArrayElements(message, nullptr);
    jsize sig_len = env->GetArrayLength(signature);
    jbyte* sig = env->GetByteArrayElements(signature, nullptr);
    jsize pub_len = env->GetArrayLength(publicKey);
    jbyte* pub = env->GetByteArrayElements(publicKey, nullptr);

    bool valid = cathedral_mldsa_verify(
        reinterpret_cast<uint8_t*>(msg), msg_len,
        reinterpret_cast<uint8_t*>(sig), sig_len,
        reinterpret_cast<uint8_t*>(pub), pub_len
    );

    env->ReleaseByteArrayElements(message, msg, JNI_ABORT);
    env->ReleaseByteArrayElements(signature, sig, JNI_ABORT);
    env->ReleaseByteArrayElements(publicKey, pub, JNI_ABORT);

    return valid ? JNI_TRUE : JNI_FALSE;
}

// ============================================================================
// ML-KEM (Kyber) — FIPS 203
// ============================================================================

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_MLKEM_nativeGenerateKeypair(
    JNIEnv* env,
    jclass clazz,
    jint level
) {
    CathedralMlKemKeyPair kp = cathedral_mlkem_generate_keypair(level);
    if (kp.public_key == nullptr || kp.private_key == nullptr) {
        throwCathedralException(env, "Falha ao gerar par de chaves ML-KEM");
        return nullptr;
    }

    size_t total_len = sizeof(uint32_t) + kp.public_key_len + sizeof(uint32_t) + kp.private_key_len;
    std::vector<uint8_t> buffer(total_len);
    size_t offset = 0;

    uint32_t pub_len = kp.public_key_len;
    memcpy(buffer.data() + offset, &pub_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, kp.public_key, pub_len);
    offset += pub_len;

    uint32_t priv_len = kp.private_key_len;
    memcpy(buffer.data() + offset, &priv_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, kp.private_key, priv_len);

    cathedral_free(kp.public_key);
    cathedral_free(kp.private_key);

    jbyteArray result = env->NewByteArray(static_cast<jsize>(total_len));
    env->SetByteArrayRegion(result, 0, total_len, reinterpret_cast<const jbyte*>(buffer.data()));
    return result;
}

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_MLKEM_nativeEncapsulate(
    JNIEnv* env,
    jclass clazz,
    jbyteArray publicKey,
    jint level
) {
    jsize pub_len = env->GetArrayLength(publicKey);
    jbyte* pub = env->GetByteArrayElements(publicKey, nullptr);

    size_t ciphertext_len, shared_secret_len;
    uint8_t* ciphertext;
    uint8_t* shared_secret;
    cathedral_mlkem_encapsulate(
        reinterpret_cast<uint8_t*>(pub), pub_len, level,
        &ciphertext, &ciphertext_len,
        &shared_secret, &shared_secret_len
    );

    env->ReleaseByteArrayElements(publicKey, pub, JNI_ABORT);

    if (ciphertext == nullptr || shared_secret == nullptr) {
        throwCathedralException(env, "Falha no encapsulamento ML-KEM");
        return nullptr;
    }

    // Retorna [ciphertext_len, ciphertext, shared_secret_len, shared_secret]
    size_t total_len = sizeof(uint32_t) + ciphertext_len + sizeof(uint32_t) + shared_secret_len;
    std::vector<uint8_t> buffer(total_len);
    size_t offset = 0;

    uint32_t ct_len = static_cast<uint32_t>(ciphertext_len);
    memcpy(buffer.data() + offset, &ct_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, ciphertext, ct_len);
    offset += ct_len;

    uint32_t ss_len = static_cast<uint32_t>(shared_secret_len);
    memcpy(buffer.data() + offset, &ss_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, shared_secret, ss_len);

    cathedral_free(ciphertext);
    cathedral_free(shared_secret);

    jbyteArray result = env->NewByteArray(static_cast<jsize>(total_len));
    env->SetByteArrayRegion(result, 0, total_len, reinterpret_cast<const jbyte*>(buffer.data()));
    return result;
}

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_MLKEM_nativeDecapsulate(
    JNIEnv* env,
    jclass clazz,
    jbyteArray ciphertext,
    jbyteArray privateKey,
    jint level
) {
    jsize ct_len = env->GetArrayLength(ciphertext);
    jbyte* ct = env->GetByteArrayElements(ciphertext, nullptr);
    jsize priv_len = env->GetArrayLength(privateKey);
    jbyte* priv = env->GetByteArrayElements(privateKey, nullptr);

    size_t shared_secret_len;
    uint8_t* shared_secret = cathedral_mlkem_decapsulate(
        reinterpret_cast<uint8_t*>(ct), ct_len,
        reinterpret_cast<uint8_t*>(priv), priv_len, level,
        &shared_secret_len
    );

    env->ReleaseByteArrayElements(ciphertext, ct, JNI_ABORT);
    env->ReleaseByteArrayElements(privateKey, priv, JNI_ABORT);

    if (shared_secret == nullptr) {
        throwCathedralException(env, "Falha no decapsulamento ML-KEM");
        return nullptr;
    }

    jbyteArray result = env->NewByteArray(static_cast<jsize>(shared_secret_len));
    env->SetByteArrayRegion(result, 0, shared_secret_len, reinterpret_cast<const jbyte*>(shared_secret));
    cathedral_free(shared_secret);
    return result;
}

// ============================================================================
// SLH-DSA (SPHINCS+) — FIPS 205
// ============================================================================

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_SLHDSA_nativeGenerateKeypair(
    JNIEnv* env,
    jclass clazz,
    jint level
) {
    CathedralSlhDsaKeyPair kp = cathedral_slhdsa_generate_keypair(level);
    if (kp.public_key == nullptr || kp.private_key == nullptr) {
        throwCathedralException(env, "Falha ao gerar par de chaves SLH-DSA");
        return nullptr;
    }

    size_t total_len = sizeof(uint32_t) + kp.public_key_len + sizeof(uint32_t) + kp.private_key_len;
    std::vector<uint8_t> buffer(total_len);
    size_t offset = 0;

    uint32_t pub_len = kp.public_key_len;
    memcpy(buffer.data() + offset, &pub_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, kp.public_key, pub_len);
    offset += pub_len;

    uint32_t priv_len = kp.private_key_len;
    memcpy(buffer.data() + offset, &priv_len, sizeof(uint32_t));
    offset += sizeof(uint32_t);
    memcpy(buffer.data() + offset, kp.private_key, priv_len);

    cathedral_free(kp.public_key);
    cathedral_free(kp.private_key);

    jbyteArray result = env->NewByteArray(static_cast<jsize>(total_len));
    env->SetByteArrayRegion(result, 0, total_len, reinterpret_cast<const jbyte*>(buffer.data()));
    return result;
}

JNIEXPORT jbyteArray JNICALL
Java_cathedral_pqc_SLHDSA_nativeSign(
    JNIEnv* env,
    jclass clazz,
    jbyteArray message,
    jbyteArray privateKey
) {
    jsize msg_len = env->GetArrayLength(message);
    jbyte* msg = env->GetByteArrayElements(message, nullptr);
    jsize priv_len = env->GetArrayLength(privateKey);
    jbyte* priv = env->GetByteArrayElements(privateKey, nullptr);

    size_t sig_len;
    uint8_t* signature = cathedral_slhdsa_sign(
        reinterpret_cast<uint8_t*>(msg), msg_len,
        reinterpret_cast<uint8_t*>(priv), priv_len,
        &sig_len
    );

    env->ReleaseByteArrayElements(message, msg, JNI_ABORT);
    env->ReleaseByteArrayElements(privateKey, priv, JNI_ABORT);

    if (signature == nullptr) {
        throwCathedralException(env, "Falha na assinatura SLH-DSA");
        return nullptr;
    }

    jbyteArray result = env->NewByteArray(static_cast<jsize>(sig_len));
    env->SetByteArrayRegion(result, 0, sig_len, reinterpret_cast<const jbyte*>(signature));
    cathedral_free(signature);
    return result;
}

JNIEXPORT jboolean JNICALL
Java_cathedral_pqc_SLHDSA_nativeVerify(
    JNIEnv* env,
    jclass clazz,
    jbyteArray message,
    jbyteArray signature,
    jbyteArray publicKey
) {
    jsize msg_len = env->GetArrayLength(message);
    jbyte* msg = env->GetByteArrayElements(message, nullptr);
    jsize sig_len = env->GetArrayLength(signature);
    jbyte* sig = env->GetByteArrayElements(signature, nullptr);
    jsize pub_len = env->GetArrayLength(publicKey);
    jbyte* pub = env->GetByteArrayElements(publicKey, nullptr);

    bool valid = cathedral_slhdsa_verify(
        reinterpret_cast<uint8_t*>(msg), msg_len,
        reinterpret_cast<uint8_t*>(sig), sig_len,
        reinterpret_cast<uint8_t*>(pub), pub_len
    );

    env->ReleaseByteArrayElements(message, msg, JNI_ABORT);
    env->ReleaseByteArrayElements(signature, sig, JNI_ABORT);
    env->ReleaseByteArrayElements(publicKey, pub, JNI_ABORT);

    return valid ? JNI_TRUE : JNI_FALSE;
}

// ============================================================================
// Hybrid Certificate — Ed25519 + ML-DSA
// ============================================================================

JNIEXPORT jbyteArray JNICALL
Java_cathedral_HybridCertificate_nativeGenerateHybridCertificate(
    JNIEnv* env,
    jclass clazz,
    jstring id,
    jstring agentId,
    jbyteArray ed25519PublicKey,
    jbyteArray ed25519PrivateKey,
    jbyteArray mlDsaPublicKey,
    jbyteArray mlDsaPrivateKey,
    jstring issuer,
    jlong validFrom,
    jlong validUntil
) {
    const char* idStr = env->GetStringUTFChars(id, nullptr);
    const char* agentStr = env->GetStringUTFChars(agentId, nullptr);
    const char* issuerStr = env->GetStringUTFChars(issuer, nullptr);

    jsize edPubLen = env->GetArrayLength(ed25519PublicKey);
    jbyte* edPub = env->GetByteArrayElements(ed25519PublicKey, nullptr);
    jsize edPrivLen = env->GetArrayLength(ed25519PrivateKey);
    jbyte* edPriv = env->GetByteArrayElements(ed25519PrivateKey, nullptr);
    jsize mlPubLen = env->GetArrayLength(mlDsaPublicKey);
    jbyte* mlPub = env->GetByteArrayElements(mlDsaPublicKey, nullptr);
    jsize mlPrivLen = env->GetArrayLength(mlDsaPrivateKey);
    jbyte* mlPriv = env->GetByteArrayElements(mlDsaPrivateKey, nullptr);

    size_t outLen;
    uint8_t* cert = cathedral_hybrid_certificate_generate(
        idStr, agentStr,
        reinterpret_cast<uint8_t*>(edPub), edPubLen,
        reinterpret_cast<uint8_t*>(edPriv), edPrivLen,
        reinterpret_cast<uint8_t*>(mlPub), mlPubLen,
        reinterpret_cast<uint8_t*>(mlPriv), mlPrivLen,
        issuerStr, validFrom, validUntil,
        &outLen
    );

    env->ReleaseStringUTFChars(id, idStr);
    env->ReleaseStringUTFChars(agentId, agentStr);
    env->ReleaseStringUTFChars(issuer, issuerStr);
    env->ReleaseByteArrayElements(ed25519PublicKey, edPub, JNI_ABORT);
    env->ReleaseByteArrayElements(ed25519PrivateKey, edPriv, JNI_ABORT);
    env->ReleaseByteArrayElements(mlDsaPublicKey, mlPub, JNI_ABORT);
    env->ReleaseByteArrayElements(mlDsaPrivateKey, mlPriv, JNI_ABORT);

    if (cert == nullptr) {
        throwCathedralException(env, "Falha ao gerar certificado híbrido");
        return nullptr;
    }

    jbyteArray result = env->NewByteArray(static_cast<jsize>(outLen));
    env->SetByteArrayRegion(result, 0, outLen, reinterpret_cast<const jbyte*>(cert));
    cathedral_free(cert);
    return result;
}

JNIEXPORT jboolean JNICALL
Java_cathedral_HybridCertificate_nativeVerifyHybridCertificate(
    JNIEnv* env,
    jclass clazz,
    jbyteArray certificate,
    jbyteArray ed25519PublicKey,
    jbyteArray mlDsaPublicKey
) {
    jsize certLen = env->GetArrayLength(certificate);
    jbyte* cert = env->GetByteArrayElements(certificate, nullptr);
    jsize edPubLen = env->GetArrayLength(ed25519PublicKey);
    jbyte* edPub = env->GetByteArrayElements(ed25519PublicKey, nullptr);
    jsize mlPubLen = env->GetArrayLength(mlDsaPublicKey);
    jbyte* mlPub = env->GetByteArrayElements(mlDsaPublicKey, nullptr);

    bool valid = cathedral_hybrid_certificate_verify(
        reinterpret_cast<uint8_t*>(cert), certLen,
        reinterpret_cast<uint8_t*>(edPub), edPubLen,
        reinterpret_cast<uint8_t*>(mlPub), mlPubLen
    );

    env->ReleaseByteArrayElements(certificate, cert, JNI_ABORT);
    env->ReleaseByteArrayElements(ed25519PublicKey, edPub, JNI_ABORT);
    env->ReleaseByteArrayElements(mlDsaPublicKey, mlPub, JNI_ABORT);

    return valid ? JNI_TRUE : JNI_FALSE;
}

} // extern "C"
