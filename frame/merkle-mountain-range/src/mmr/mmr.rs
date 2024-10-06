// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
	mmr::{
		storage::{OffchainStorage, RuntimeStorage, Storage},
		Hasher, Node, NodeOf,
	},
	primitives::{self, Error, NodeIndex},
	Config, HashOf, HashingOf,
};
use sp_mmr_primitives::{mmr_lib, mmr_lib::MMRStoreReadOps, utils::NodesUtils, LeafIndex};
use sp_std::prelude::*;

/// Stateless verification of the proof for a batch of leaves.
/// Note, the leaves should be sorted such that corresponding leaves and leaf indices have the
/// same position in both the `leaves` vector and the `leaf_indices` vector contained in the
/// [primitives::LeafProof]
pub fn verify_leaves_proof<H, L>(
	root: H::Output,
	leaves: Vec<Node<H, L>>,
	proof: primitives::LeafProof<H::Output>,
) -> Result<bool, Error>
where
	H: sp_runtime::traits::Hash,
	L: primitives::FullLeaf,
{
	let size = NodesUtils::new(proof.leaf_count).size();

	if leaves.len() != proof.leaf_indices.len() {
		return Err(Error::Verify.log_debug("Proof leaf_indices not same length with leaves"))
	}

	let leaves_and_position_data = proof
		.leaf_indices
		.into_iter()
		.map(|index| mmr_lib::leaf_index_to_pos(index))
		.zip(leaves.into_iter())
		.collect();

	let p = mmr_lib::MerkleProof::<Node<H, L>, Hasher<H, L>>::new(
		size,
		proof.items.into_iter().map(Node::Hash).collect(),
	);
	p.verify(Node::Hash(root), leaves_and_position_data)
		.map_err(|e| Error::Verify.log_debug(e))
}

/// A wrapper around an MMR library to expose limited functionality.
///
/// Available functions depend on the storage kind ([Runtime](crate::mmr::storage::RuntimeStorage)
/// vs [Off-chain](crate::mmr::storage::OffchainStorage)).
pub struct Mmr<StorageType, T, I, L>
where
	T: Config<I>,
	I: 'static,
	L: primitives::FullLeaf,
	Storage<StorageType, T, I, L>:
		MMRStoreReadOps<NodeOf<T, I, L>> + mmr_lib::MMRStoreWriteOps<NodeOf<T, I, L>>,
{
	mmr: mmr_lib::MMR<NodeOf<T, I, L>, Hasher<HashingOf<T, I>, L>, Storage<StorageType, T, I, L>>,
	leaves: NodeIndex,
}

impl<StorageType, T, I, L> Mmr<StorageType, T, I, L>
where
	T: Config<I>,
	I: 'static,
	L: primitives::FullLeaf,
	Storage<StorageType, T, I, L>:
		MMRStoreReadOps<NodeOf<T, I, L>> + mmr_lib::MMRStoreWriteOps<NodeOf<T, I, L>>,
{
	/// Create a pointer to an existing MMR with given number of leaves.
	pub fn new(leaves: NodeIndex) -> Self {
		let size = NodesUtils::new(leaves).size();
		Self { mmr: mmr_lib::MMR::new(size, Default::default()), leaves }
	}

	/// Verify proof for a set of leaves.
	/// Note, the leaves should be sorted such that corresponding leaves and leaf indices have
	/// the same position in both the `leaves` vector and the `leaf_indices` vector contained in the
	/// [primitives::LeafProof]
	pub fn verify_leaves_proof(
		&self,
		leaves: Vec<L>,
		proof: primitives::LeafProof<HashOf<T, I>>,
	) -> Result<bool, Error> {
		let p = mmr_lib::MerkleProof::<NodeOf<T, I, L>, Hasher<HashingOf<T, I>, L>>::new(
			self.mmr.mmr_size(),
			proof.items.into_iter().map(Node::Hash).collect(),
		);

		if leaves.len() != proof.leaf_indices.len() {
			return Err(Error::Verify.log_debug("Proof leaf_indices not same length with leaves"))
		}

		let leaves_positions_and_data = proof
			.leaf_indices
			.into_iter()
			.map(|index| mmr_lib::leaf_index_to_pos(index))
			.zip(leaves.into_iter().map(|leaf| Node::Data(leaf)))
			.collect();
		let root = self.mmr.get_root().map_err(|e| Error::GetRoot.log_error(e))?;
		p.verify(root, leaves_positions_and_data)
			.map_err(|e| Error::Verify.log_debug(e))
	}

	pub fn verify_ancestry_proof(
		&self,
		ancestry_proof: primitives::AncestryProof<HashOf<T, I>>,
	) -> Result<bool, Error> {
		let prev_peaks_proof =
			mmr_lib::NodeMerkleProof::<NodeOf<T, I, L>, Hasher<HashingOf<T, I>, L>>::new(
				self.mmr.mmr_size(),
				ancestry_proof
					.items
					.into_iter()
					.map(|(index, hash)| (index, Node::Hash(hash)))
					.collect(),
			);

		let raw_ancestry_proof = mmr_lib::AncestryProof::<
			NodeOf<T, I, L>,
			Hasher<HashingOf<T, I>, L>,
		> {
			prev_peaks: ancestry_proof
				.prev_peaks
				.into_iter()
				.map(|hash| Node::Hash(hash))
				.collect(),
			prev_mmr_size: mmr_lib::helper::leaf_index_to_mmr_size(ancestry_proof.prev_leaf_count - 1),
			prev_peaks_proof: prev_peaks_proof,
		};

		let prev_root = mmr_lib::ancestry_proof::bagging_peaks_hashes::<
			NodeOf<T, I, L>,
			Hasher<HashingOf<T, I>, L>,
		>(raw_ancestry_proof.prev_peaks.clone())
		.map_err(|e| Error::Verify.log_debug(e))?;
		let root = self.mmr.get_root().map_err(|e| Error::GetRoot.log_error(e))?;
		raw_ancestry_proof
			.verify_ancestor(root, prev_root)
			.map_err(|e| Error::Verify.log_debug(e))
	}

	/// Return the internal size of the MMR (number of nodes).
	#[cfg(test)]
	pub fn size(&self) -> NodeIndex {
		self.mmr.mmr_size()
	}
}

/// Runtime specific MMR functions.
impl<T, I, L> Mmr<RuntimeStorage, T, I, L>
where
	T: Config<I>,
	I: 'static,
	L: primitives::FullLeaf,
{
	/// Push another item to the MMR.
	///
	/// Returns element position (index) in the MMR.
	pub fn push(&mut self, leaf: L) -> Option<NodeIndex> {
		let position =
			self.mmr.push(Node::Data(leaf)).map_err(|e| Error::Push.log_error(e)).ok()?;

		self.leaves += 1;

		Some(position)
	}

	/// Commit the changes to underlying storage, return current number of leaves and
	/// calculate the new MMR's root hash.
	pub fn finalize(mut self) -> Result<(NodeIndex, HashOf<T, I>), Error> {
		let root = self.mmr.get_root().map_err(|e| Error::GetRoot.log_error(e))?;
		self.mmr.commit().map_err(|e| Error::Commit.log_error(e))?;
		Ok((self.leaves, root.hash()))
	}
}

/// Off-chain specific MMR functions.
impl<T, I, L> Mmr<OffchainStorage, T, I, L>
where
	T: Config<I>,
	I: 'static,
	L: primitives::FullLeaf + codec::Decode,
{
	/// Generate a proof for given leaf indices.
	///
	/// Proof generation requires all the nodes (or their hashes) to be available in the storage.
	/// (i.e. you can't run the function in the pruned storage).
	pub fn generate_proof(
		&self,
		leaf_indices: Vec<NodeIndex>,
	) -> Result<(Vec<L>, primitives::LeafProof<HashOf<T, I>>), Error> {
		let positions = leaf_indices
			.iter()
			.map(|index| mmr_lib::leaf_index_to_pos(*index))
			.collect::<Vec<_>>();
		let store = <Storage<OffchainStorage, T, I, L>>::default();
		let leaves = positions
			.iter()
			.map(|pos| match store.get_elem(*pos) {
				Ok(Some(Node::Data(leaf))) => Ok(leaf),
				e => Err(Error::LeafNotFound.log_debug(e)),
			})
			.collect::<Result<Vec<_>, Error>>()?;

		let leaf_count = self.leaves;
		self.mmr
			.gen_proof(positions)
			.map_err(|e| Error::GenerateProof.log_error(e))
			.map(|p| primitives::LeafProof {
				leaf_indices,
				leaf_count,
				items: p.proof_items().iter().map(|x| x.hash()).collect(),
			})
			.map(|p| (leaves, p))
	}

	pub fn generate_ancestry_proof(
		&self,
		prev_leaf_count: LeafIndex,
	) -> Result<primitives::AncestryProof<HashOf<T, I>>, Error> {
		let prev_mmr_size = NodesUtils::new(prev_leaf_count).size();
		let raw_ancestry_proof = self
			.mmr
			.gen_ancestry_proof(prev_mmr_size)
			.map_err(|e| Error::GenerateProof.log_error(e))?;

		Ok(primitives::AncestryProof {
			prev_peaks: raw_ancestry_proof.prev_peaks.into_iter().map(|p| p.hash()).collect(),
			prev_leaf_count,
			leaf_count: self.leaves,
			items: raw_ancestry_proof
				.prev_peaks_proof
				.proof_items()
				.iter()
				.map(|(index, item)| (*index, item.hash()))
				.collect(),
		})
	}
}
