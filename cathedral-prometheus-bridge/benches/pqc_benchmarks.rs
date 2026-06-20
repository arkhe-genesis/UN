use criterion::{criterion_group, criterion_main, Criterion};
use cathedral_sdk_rs::crypto::{SigningKeyWrapper, VerifyingKeyWrapper, CryptoFactory};
use common::crypto_config::{CryptoConfig, SignatureAlgorithm};

fn bench_pqc_signatures(c: &mut Criterion) {
    let config_ed25519 = CryptoConfig {
        signature_algorithm: SignatureAlgorithm::Ed25519,
        ..Default::default()
    };
    let config_mldsa = CryptoConfig {
        signature_algorithm: SignatureAlgorithm::MlDsa,
        ..Default::default()
    };

    let factory_ed25519 = CryptoFactory::new(config_ed25519);
    let factory_mldsa = CryptoFactory::new(config_mldsa);

    let sk_ed25519 = factory_ed25519.generate_signing_key().unwrap();
    let sk_mldsa = factory_mldsa.generate_signing_key().unwrap();

    let msg = vec![0u8; 1024]; // mensagem de 1KB

    let mut group = c.benchmark_group("signature_generation");
    group.bench_function("Ed25519", |b| {
        b.iter(|| sk_ed25519.sign(&msg).unwrap())
    });
    group.bench_function("ML-DSA", |b| {
        b.iter(|| sk_mldsa.sign(&msg).unwrap())
    });
    group.finish();

    // Verificação
    let vk_ed25519 = match &sk_ed25519 {
        SigningKeyWrapper::Ed25519(sk) => VerifyingKeyWrapper::Ed25519(ed25519_dalek::VerifyingKey::from(sk)),
        _ => unreachable!(),
    };
    let vk_mldsa = match &sk_mldsa {
        SigningKeyWrapper::MlDsa(_) => {
            let (pk, _) = pqcrypto_dilithium::dilithium3::keypair();
            VerifyingKeyWrapper::MlDsa(pk)
        }
        _ => unreachable!(),
    };

    let sig_ed25519 = sk_ed25519.sign(&msg).unwrap();
    let sig_mldsa = sk_mldsa.sign(&msg).unwrap();

    let mut group = c.benchmark_group("signature_verification");
    group.bench_function("Ed25519", |b| {
        b.iter(|| vk_ed25519.verify(&msg, &sig_ed25519).unwrap())
    });
    group.bench_function("ML-DSA", |b| {
        b.iter(|| vk_mldsa.verify(&msg, &sig_mldsa).unwrap())
    });
    group.finish();
}

criterion_group!(benches, bench_pqc_signatures);
criterion_main!(benches);
