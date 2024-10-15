use crate::types::AttributedId;
use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::Get;

pub trait Did<AccountId, BlockNumber, Moment, MaxNameLength, MaxValueLength, Error> 
where
	MaxNameLength: Get<u32>,
    MaxValueLength: Get<u32>,
{
	fn is_owner(identity: &AccountId, actual_owner: &AccountId) -> Result<(), Error>;
	fn identity_owner(identity: &AccountId) -> AccountId;
	fn create_attribute(
		who: &AccountId,
		identity: &AccountId,
		name: &[u8],
		value: &[u8],
		valid_for: Option<BlockNumber>,
	) -> Result<(), Error>;
	fn reset_attribute(who: &AccountId, identity: &AccountId, name: &[u8]) -> DispatchResult;
	fn valid_attribute(identity: &AccountId, name: &[u8], value: &[u8]) -> DispatchResult;
	fn attribute_and_id(
		identity: &AccountId,
		name: &[u8],
	) -> Option<AttributedId<BlockNumber, Moment, MaxNameLength, MaxValueLength>>;
}
