// #![allow(unused_imports)]
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

use frame_support::pallet_prelude::Get;
use frame_support::sp_runtime::BoundedVec;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
/// Attributes or properties that make an DID.

#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, MaxEncodedLen, TypeInfo,
)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Attribute<BlockNumber, Moment, MaxNameLength, MaxValueLength>
where
	MaxNameLength: Get<u32>,
	MaxValueLength: Get<u32>,
{
	pub name: BoundedVec<u8, MaxNameLength>,
	pub value: BoundedVec<u8, MaxValueLength>,
	pub validity: BlockNumber,
	pub creation: Moment,
	pub nonce: u64,
}

pub type AttributedId<BlockNumber, Moment, MaxNameLength, MaxValueLength> =
	(Attribute<BlockNumber, Moment, MaxNameLength, MaxValueLength>, [u8; 32]);
