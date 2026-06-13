use arkhe_cognitive_core::CognitiveCore;
use arya_stark::Gradient;
use sealy::{Ciphertext, Encryptor, KeyGenerator, Plaintext, ToBytes};
use gradient::create_fhe_context;

#[test]
fn test_scalability_multiple_orbs() {
    let core = CognitiveCore::new();
    let num_orbs = 100;

    let mut local_gradients = Vec::new();
    let mut encrypted_gradients = Vec::new();

    let context = create_fhe_context();
    let mut keygen = KeyGenerator::new(&context).unwrap();
    let pk = keygen.create_public_key();
    let encryptor = Encryptor::with_public_key(&context, &pk).unwrap();

    let pt = Plaintext::new().unwrap();
    let ct = encryptor.encrypt(&pt).unwrap();
    let ct_bytes = ct.as_bytes().unwrap();

    for _ in 0..num_orbs {
        local_gradients.push(Gradient {
            data: vec![0.5; 100],
        });

        encrypted_gradients.push(ct_bytes.clone());
    }

    core.cycle(local_gradients, encrypted_gradients);
}
