#[cfg(target_os = "zkvm")]
use risc0_zkvm::guest;
use risc0_zkvm::{
    declare_syscall,
    sha::{
        rust_crypto::{Digest as _, Sha256},
        Digest,
    },
};

declare_syscall!(
    /// RISC0 syscall for providing oracle access to a vector committed to by the host.
    pub SYS_VECTOR_ORACLE);

/*use risc0_zkvm::{
    default_prover,
    sha::{
        rust_crypto::{Digest as _, Sha256},
        Digest,
    },
    ExecutorEnv,
};*/
use serde::{Deserialize, Serialize};

//use merkle_light::merkle::MerkleTree;

use merkle_light::{
    hash::{Algorithm, Hashable},
    merkle, proof,
};
use std::{hash::Hasher, marker::PhantomData, ops::Deref};

pub struct MerkleTree<Element>
where
    Element: Hashable<ShaHasher>,
{
    tree: merkle::MerkleTree<Node, ShaHasher>,
    elements: Vec<Element>,
}

impl<Element> MerkleTree<Element>
where
    Element: Hashable<ShaHasher>,
{
    pub fn new(elements: Vec<Element>) -> Self {
        Self {
            tree: merkle::MerkleTree::<_, ShaHasher>::from_data(elements.iter()),
            elements,
        }
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn prove(&self, i: usize) -> Proof<Element> {
        self.tree.gen_proof(i).into()
    }
}

// Implement Deref so that all the methods on the wrapped type are accessible.
impl<Element> Deref for MerkleTree<Element>
where
    Element: Hashable<ShaHasher>,
{
    type Target = merkle::MerkleTree<Node, ShaHasher>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

#[derive(Default)]
pub struct ShaHasher(Sha256);
pub type Node = Digest;

impl Hasher for ShaHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    fn finish(&self) -> u64 {
        unimplemented!("finish is not implemented for merkletree hashers");
    }
}

impl Algorithm<Node> for ShaHasher {
    fn hash(&mut self) -> Node {
        Node::try_from(self.0.finalize_reset().as_slice()).unwrap()
    }
}

pub fn u64_to_u8_32_array(num: u64) -> [u8; 32] {
    let mut arr = [0u8; 32];
    arr[..8].copy_from_slice(&num.to_be_bytes());
    arr
}

/// Wrapper for the `merkle_light` inclusion proof. Includes an improved API for
/// verifying that a proof supports serialization and references an expected
/// element and position.
#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "(Vec<Node>, Vec<bool>)", into = "(Vec<Node>, Vec<bool>)")]
pub struct Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    inner: proof::Proof<Node>,
    phantom_elem: PhantomData<Element>,
}

impl<Element> Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    /// Verify that the proof commits to the inclusion of the given element in a
    /// Merkle tree with the given root.
    pub fn verify(&self, root: &Node, element: &Element) -> bool {
        // Check that the root of the proof matches the provided root.
        match &self.verified_root(element) {
            Some(ref verified_root) => verified_root == root,
            None => false,
        }
    }

    /// Verify that the proof commits to the element in _some_ Merkle tree and
    /// return the calculated Merkle root.
    pub fn verified_root(&self, element: &Element) -> Option<Node> {
        // Check that the path from the leaf to the root is consistent.
        if !self.inner.validate::<ShaHasher>() {
            println!("1");
            return None;
        }

        // Check the element hashes to the leaf in the proof.
        let algorithm = &mut ShaHasher::default();
        element.hash(algorithm);
        let elem_hash = algorithm.hash();

        // Hash the element to get the leaf, and check that it matches.
        algorithm.reset();
        if algorithm.leaf(elem_hash) != self.inner.item() {
            println!("self.inner.item(): {:?}", self.inner.item());
            println!("elem_hash: {:?}", elem_hash);
            println!("algorithm.leaf(elem_hash): {:?}", algorithm.leaf(elem_hash));
            println!("2");
            return None;
        }

        Some(self.root())
    }

    /// Compute the vector index of the proven element.
    pub fn index(&self) -> usize {
        self.inner
            .path()
            .iter()
            .rfold(0, |index, bit| (index << 1) + (!*bit as usize))
    }
}

impl<Element> Clone for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom_elem: PhantomData,
        }
    }
}

impl<Element> Deref for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    type Target = proof::Proof<Node>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Element> From<proof::Proof<Node>> for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn from(inner: proof::Proof<Node>) -> Self {
        Self {
            inner,
            phantom_elem: PhantomData,
        }
    }
}

// From tuple representation provided to enable serde deserialization.
impl<Element> From<(Vec<Node>, Vec<bool>)> for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn from(tuple: (Vec<Node>, Vec<bool>)) -> Self {
        proof::Proof::new(tuple.0, tuple.1).into()
    }
}

// Into tuple representation provided to enable serde deserialization.
impl<Element> Into<(Vec<Node>, Vec<bool>)> for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn into(self) -> (Vec<Node>, Vec<bool>) {
        (self.inner.lemma().to_vec(), self.inner.path().to_vec())
    }
}
