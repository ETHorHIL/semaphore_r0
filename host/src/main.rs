use std::{hash, ptr::hash};

// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{SEMAPHORE_GUEST_ELF, SEMAPHORE_GUEST_ID};

use merkletree::*;
use risc0_zkvm::{
    default_prover,
    sha::{
        rust_crypto::{Digest as _, Sha256},
        Digest,
    },
    ExecutorEnv,
};
use serde::{Deserialize, Serialize};

use merkle_light::merkle::MerkleTree;
//use merkle_light::proof;
use merkle_light::{
    hash::{Algorithm, Hashable},
    proof::Proof,
};

// convenience fn to hash two u64 values
fn hash2_u64(a: u64, b: u64) -> [u8; 32] {
    let mut res = Vec::new();
    res.extend_from_slice(&u64_to_u8_32_array(a));
    res.extend_from_slice(&u64_to_u8_32_array(b));

    Sha256::digest(id_nullifier_trapdoor_as_bytes)
        .try_into()
        .unwrap()
}

fn main() {
    env_logger::init();
    // public values externally generated
    let external_nullifier = u64_to_u8_32_array(42); // this is a topic, can he any value
    let signal_hash = u64_to_u8_32_array(99); // consistency should be checked by the smart contract against the msg hash

    // private inouts, can just pick
    let identity_nullifier = u64_to_u8_32_array(111222333);
    let identity_trapdoor = u64_to_u8_32_array(222333444);

    let mut id_nullifier_trapdoor_as_bytes = Vec::new();
    id_nullifier_trapdoor_as_bytes.extend_from_slice(&identity_nullifier);
    id_nullifier_trapdoor_as_bytes.extend_from_slice(&identity_trapdoor);

    let digest = Sha256::digest(id_nullifier_trapdoor_as_bytes);
    //let digest = Digest::try_from(digest.as_slice()).unwrap();
    let identity_commitment = Sha256::digest(digest);
    //let identity_commitment = Digest::try_from(identity_commitment).unwrap();

    // private values, determinde by merkle tree
    //let identity_commitment = Digest::try_from(identity_commitment).unwrap();
    println!("identity_commitment: {:?}", identity_commitment);
    let leaf_index: usize = 1;

    let mut external_nullifier_identity_nullifier_as_bytes = Vec::new();
    external_nullifier_identity_nullifier_as_bytes.extend_from_slice(&external_nullifier);
    external_nullifier_identity_nullifier_as_bytes.extend_from_slice(&identity_nullifier);
    let nullifier_hash: [u8; 32] = Sha256::digest(external_nullifier_identity_nullifier_as_bytes)
        .try_into()
        .unwrap();
    //let nullifier_hash = Digest::try_from(nullifier_hash.as_slice()).unwrap();

    // Construct a tree, insert the identity and retrieve the root and merkle proof
    let mut tree_leafes = vec![[0u8; 32]; 16];
    tree_leafes[leaf_index] = identity_commitment.try_into().unwrap();

    let t = merkletree::MerkleTree::<[u8; 32]>::new(tree_leafes);
    let root: [u8; 32] = t.root().try_into().unwrap();
    println!("{:?}", root);

    let proof = t.prove(leaf_index);
    println!("{:?}", proof);
    let verify = proof.verify(
        &Digest::from(root),
        &identity_commitment.try_into().unwrap(),
    );
    println!("{:?}", verify);

    // Inputs to the circuit:
    // private inputs: identity_nullifier, identity_trapdoor, leaf_index, tree_siblings
    // public inputs: signal_hash, external_nullifier, root, nullifier_hash
    let private_inputs = (identity_nullifier, identity_trapdoor, proof);

    let public_inputs = (signal_hash, external_nullifier, root, nullifier_hash);

    let env = ExecutorEnv::builder()
        .write(&private_inputs)
        .unwrap()
        .write(&public_inputs)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove_elf(env, SEMAPHORE_GUEST_ELF).unwrap();

    // TODO: Implement code for retrieving receipt journal here.

    // For example:
    let _output: bool = receipt.journal.decode().unwrap();
    println!("{}", _output);

    // Optional: Verify receipt to confirm that recipients will also be able to
    // verify your receipt
    receipt.verify(SEMAPHORE_GUEST_ID).unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}