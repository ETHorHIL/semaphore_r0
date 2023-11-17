#![no_main]

use merkletree::Proof;
use risc0_zkvm::{guest::env, sha::Digest};
use sha2::{Digest as _, Sha256};
risc0_zkvm::guest::entry!(main);

fn calculate_secret(identity_nullifier: [u8; 32], identity_trapdoor: [u8; 32]) -> [u8; 32] {
    let mut input = Vec::new();
    input.extend_from_slice(&identity_nullifier);
    input.extend_from_slice(&identity_trapdoor);

    let digest = Sha256::digest(input);
    let digest: [u8; 32] = digest.try_into().unwrap();
    digest
}

fn calculate_identity_commitment(secret: [u8; 32]) -> [u8; 32] {
    let digest = Sha256::digest(secret);
    let res: [u8; 32] = digest.try_into().unwrap();
    res
}

fn calculate_nullifier_hash(
    external_nullifier: [u8; 32],
    identity_nullifier: [u8; 32],
) -> [u8; 32] {
    let mut input = Vec::new();
    input.extend_from_slice(&external_nullifier);
    input.extend_from_slice(&identity_nullifier);

    let digest = Sha256::digest(input);
    digest.try_into().unwrap()
}

fn semaphore(
    identity_nullifier: [u8; 32],
    identity_trapdoor: [u8; 32],
    external_nullifier: [u8; 32],
) -> [[u8; 32]; 2] {
    let secret = calculate_secret(identity_nullifier, identity_trapdoor);
    let leaf = calculate_identity_commitment(secret);

    let nullifier_hash = calculate_nullifier_hash(external_nullifier, identity_nullifier);
    //let _signal_hash_squared = signal_hash * signal_hash;
    [leaf, nullifier_hash]
}

// private inputs: identity_nullifier, identity_trapdoor, tree_siblings
type PrivateInputs = ([u8; 32], [u8; 32], Proof<[u8; 32]>);
// public inputs: signal_hash, external_nullifier, root, nullifier_hash
type PublicInputs = ([u8; 32], [u8; 32], [u8; 32], [u8; 32]);

pub fn main() {
    // read the input

    let (private_inputs, public_inputs): (PrivateInputs, PublicInputs) = env::read();
    let (identity_nullifier, identity_trapdoor, tree_siblings) = private_inputs;
    let (signal_hash, external_nullifier, root, nullifier_hash) = public_inputs;

    let circuit_output = semaphore(identity_nullifier, identity_trapdoor, external_nullifier);
    let verify = tree_siblings.verify(
        &Digest::try_from(root.as_slice()).unwrap(),
        &circuit_output[0],
    );

    assert!(verify);
    assert!(nullifier_hash == circuit_output[1]);

    //let output: [u8; 32] = [0; 32];

    // write public output to the journal
    env::commit(&verify);
}
