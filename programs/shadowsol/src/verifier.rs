use anchor_lang::prelude::*;
use crate::groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};
use crate::verifying_key::VERIFYINGKEY;


#[derive(Accounts)]
pub struct VerifyProof {}

#[error_code]
pub enum ErrorCode {
    #[msg("Proof creation failed")]
    ProofCreationFailed,
    #[msg("Proof verification failed")]
    ProofVerificationFailed,
    #[msg("Invalid proof length")]
    InvalidProofLength,
    #[msg("Invalid public inputs length")]
    InvalidPublicInputsLength,
    #[msg("Invalid zk-proof provided")]
    InvalidProof,
}

/// Основная функция проверки доказательства
pub fn verify_proof_logic(
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
) -> Result<()> {

    const N: usize = 4;
    let expected_bytes = N * 32;
    require!(
        public_inputs.len() == expected_bytes,
        ErrorCode::InvalidPublicInputsLength
    );

    let mut public_inputs_arr = [[0u8; 32]; N];
    for (i, chunk) in public_inputs.chunks(32).enumerate() {
        public_inputs_arr[i].copy_from_slice(chunk);
    }

    require!(proof.len() == 256, ErrorCode::InvalidProofLength);

    let proof_a: [u8; 64] = proof[0..64]
        .try_into()
        .map_err(|_| ErrorCode::ProofCreationFailed)?;
    let proof_b: [u8; 128] = proof[64..192]
        .try_into()
        .map_err(|_| ErrorCode::ProofCreationFailed)?;
    let proof_c: [u8; 64] = proof[192..256]
        .try_into()
        .map_err(|_| ErrorCode::ProofCreationFailed)?;

    let mut verifier = Groth16Verifier::new(
        &proof_a,
        &proof_b,
        &proof_c,
        &public_inputs_arr,
        &VERIFYINGKEY,
    )
    .map_err(|_| ErrorCode::ProofCreationFailed)?;

    let is_valid = verifier.verify().map_err(|_| ErrorCode::ProofVerificationFailed)?;

    require!(is_valid, ErrorCode::InvalidProof);

    Ok(())
}
