use arya_stark::{prove_gradient, verify_gradient, Gradient, GradientProof};
use sealy::{BFVEvaluator, Context, SecurityLevel, CoefficientModulusFactory, DegreeType, Ciphertext, Evaluator, FromBytes, ToBytes, BFVEncryptionParametersBuilder};

pub struct MockCoordinator {}

impl MockCoordinator {
    pub fn new() -> Self { MockCoordinator {} }
    pub fn aggregate(&self) { println!("Aggregated using MockCoordinator"); }
}

pub fn federate_gradients() {
    let coordinator = MockCoordinator::new();
    coordinator.aggregate();
}

pub fn create_fhe_context() -> Context {
    let builder = BFVEncryptionParametersBuilder::new()
        .set_poly_modulus_degree(DegreeType::D2048)
        .set_coefficient_modulus(CoefficientModulusFactory::bfv(DegreeType::D2048, SecurityLevel::TC128).unwrap())
        .set_plain_modulus_u64(1032193);
    let params = builder.build().unwrap();
    Context::new(&params, true, SecurityLevel::TC128).unwrap()
}

pub fn homomorphic_aggregation(ciphertexts: Vec<Vec<u8>>) -> Vec<u8> {
    if ciphertexts.is_empty() {
        return vec![];
    }
    let context = create_fhe_context();
    let evaluator = BFVEvaluator::new(&context).unwrap();

    let mut aggregated_ct = Ciphertext::from_bytes(&context, &ciphertexts[0]).expect("Failed to deserialize first ciphertext");

    for bytes in ciphertexts.iter().skip(1) {
        let ct = Ciphertext::from_bytes(&context, bytes).expect("Failed to deserialize ciphertext");
        evaluator.add_inplace(&mut aggregated_ct, &ct).unwrap();
    }

    aggregated_ct.as_bytes().unwrap()
}

pub fn certify_gradient(gradient: &Gradient) -> GradientProof {
    prove_gradient(gradient).expect("Proof generation failed")
}

pub fn validate_gradient(proof: &GradientProof) -> bool {
    verify_gradient(proof).expect("Proof verification failed")
}
