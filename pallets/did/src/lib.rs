#![cfg_attr(not(feature = "std"), no_std)]

/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
/// Based on pallet-DID from https://github.com/substrate-developer-hub/pallet-did
/// The DID pallet allows resolving and management for DIDs (Decentralized Identifiers).
pub use pallet::*;
pub mod did;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// pub mod weights;
// pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	// use super::WeightInfo;
	use crate::did::Did;
	pub use crate::types::*;
	pub use frame_support::traits::Time as MomentTime;
	use frame_support::{
		dispatch::DispatchResult,
		// debug,
		pallet_prelude::*,
	};

	// use scale_info::{prelude::vec::Vec, TypeInfo};
	// use sp_std::vec::Vec;

	use sp_io::hashing::blake2_256;

	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Time: MomentTime;

		#[pallet::constant]
		type MaxNameLength: Get<u32> + Default + TypeInfo;

		#[pallet::constant]
		type MaxValueLength: Get<u32> + Default + TypeInfo;
	}

	/// The attributes that belong to an identity.
	#[pallet::storage]
	#[pallet::getter(fn attribute_of)]
	pub(super) type AttributeOf<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, [u8; 32]),
		Attribute<
			BlockNumberFor<T>,
			<<T as Config>::Time as MomentTime>::Moment,
			<T as Config>::MaxNameLength,
			<T as Config>::MaxValueLength,
		>,
		ValueQuery,
	>;

	/// Attribute nonce used to generate a unique hash even if the attribute is deleted and
	/// recreated.
	#[pallet::storage]
	#[pallet::getter(fn nonce_of)]
	/// Keeps track of adoptions events.
	pub(super) type AttributeNonce<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		(T::AccountId, BoundedVec<u8, T::MaxNameLength>),
		u64,
		ValueQuery,
	>;

	/// Identity owner.
	#[pallet::storage]
	#[pallet::getter(fn owner_of)]
	pub(super) type OwnerStore<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::AccountId>;

	/// Tracking the latest identity update.
	#[pallet::storage]
	#[pallet::getter(fn updated_by)]
	/// Keeps track of adoptions events.
	pub(super) type UpdatedBy<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		(T::AccountId, BlockNumberFor<T>, <<T as Config>::Time as MomentTime>::Moment),
	>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when an attribute has been added.
		AttributeAdded {
			identity: T::AccountId,
			name: Vec<u8>,
			value: Vec<u8>,
			valid_for: Option<BlockNumberFor<T>>,
		},
		/// Event emitted when an attribute is read successfully
		AttributeFetched { who: T::AccountId, identity: T::AccountId, name: Vec<u8> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// AccountId 'actual_owner' owns the identity.
		NotOwner,
		// reset attribute validity failed
		AttributeResetFailed,
		// Validates an attribute
		InvalidAttribute,
		// Overflow call
		Overflow,
		// Name is greater that 64
		AttributeNameExceedMax64,
		// Attribute already exist
		DuplicateNotNeeded,
		// Attribute was not found
		AttributeNotFound,
		//Name is too long
		NameTooLong,
		//Value is too long
		ValueTooLong,
	}

	impl<T: Config>
		Did<
			T::AccountId,
			BlockNumberFor<T>,
			<<T as Config>::Time as MomentTime>::Moment,
			<T as Config>::MaxNameLength,
			<T as Config>::MaxValueLength,
			Error<T>,
		> for Pallet<T>
	{
		/// Validates if the AccountId 'actual_owner' owns the identity.
		fn is_owner(identity: &T::AccountId, actual_owner: &T::AccountId) -> Result<(), Error<T>> {
			let owner = Self::identity_owner(identity);
			match owner == *actual_owner {
				true => Ok(()),
				false => Err(Error::<T>::NotOwner.into()),
			}
		}

		/// Get the identity owner if set.
		/// If never changed, returns the identity as its owner.
		fn identity_owner(identity: &T::AccountId) -> T::AccountId {
			match Self::owner_of(identity) {
				Some(id) => id,
				None => identity.clone(),
			}
		}

		/// Adds a new attribute to an identity and colects the storage fee.
		fn create_attribute(
			who: &T::AccountId,
			identity: &T::AccountId,
			name: &[u8],
			value: &[u8],
			valid_for: Option<BlockNumberFor<T>>,
		) -> Result<(), Error<T>> {
			Self::is_owner(&identity, &who)?;

			if Self::attribute_and_id(identity, name).is_some() {
				Err(Error::<T>::DuplicateNotNeeded.into())
			} else {
				let now_timestamp = T::Time::now();
				let now_block_number = <frame_system::Pallet<T>>::block_number();
				let validity: BlockNumberFor<T> = match valid_for {
					Some(blocks) => now_block_number + blocks,
					None => u32::max_value().into(),
				};

				let bounded_name_nonce: BoundedVec<u8, T::MaxNameLength> =
					name.to_vec().try_into().map_err(|_| Error::<T>::NameTooLong)?;

				let mut nonce = Self::nonce_of((&identity, bounded_name_nonce));

				let id = (&identity, name, nonce).using_encoded(blake2_256);

				let bounded_name: BoundedVec<u8, T::MaxNameLength> =
					(&name).to_vec().try_into().map_err(|_| Error::<T>::NameTooLong)?;

				let bounded_value: BoundedVec<u8, T::MaxValueLength> =
					(&value).to_vec().try_into().map_err(|_| Error::<T>::ValueTooLong)?;

				let new_attribute = Attribute {
					name: bounded_name,
					value: bounded_value,
					validity,
					creation: now_timestamp,
					nonce,
				};

				// Prevent panic overflow
				nonce = nonce.checked_add(1).ok_or(Error::<T>::Overflow)?;
				<AttributeOf<T>>::insert((&identity, &id), new_attribute);
				let bounded_name: BoundedVec<u8, T::MaxNameLength> =
					name.to_vec().try_into().map_err(|_| Error::<T>::NameTooLong)?;
				<AttributeNonce<T>>::mutate((&identity, bounded_name), |n| *n = nonce);
				<UpdatedBy<T>>::insert(identity, (who, now_block_number, now_timestamp));
				Ok(())
			}
		}

		/// Updates the attribute validity to make it expire and invalid.
		fn reset_attribute(
			who: &T::AccountId,
			identity: &T::AccountId,
			name: &[u8],
		) -> DispatchResult {
			Self::is_owner(&identity, who)?;
			// If the attribute contains_key, the latest valid block is set to the current block.
			let result = Self::attribute_and_id(identity, name);
			match result {
				Some((mut attribute, id)) => {
					attribute.validity = <frame_system::Pallet<T>>::block_number();
					<AttributeOf<T>>::mutate((&identity, id), |a| *a = attribute);
				},
				None => return Err(Error::<T>::AttributeResetFailed.into()),
			}

			// Keep track of the updates.
			<UpdatedBy<T>>::insert(
				identity,
				(who, <frame_system::Pallet<T>>::block_number(), T::Time::now()),
			);
			Ok(())
		}

		/// Validates if an attribute belongs to an identity and it has not expired.
		fn valid_attribute(identity: &T::AccountId, name: &[u8], value: &[u8]) -> DispatchResult {
			ensure!(name.len() <= 64, Error::<T>::InvalidAttribute);
			let result = Self::attribute_and_id(identity, name);

			let (attr, _) = match result {
				Some((attr, id)) => (attr, id),
				None => return Err(Error::<T>::InvalidAttribute.into()),
			};

			if (attr.validity > (<frame_system::Pallet<T>>::block_number()))
				&& (attr.value == value.to_vec())
			{
				Ok(())
			} else {
				Err(Error::<T>::InvalidAttribute.into())
			}
		}

		/// Returns the attribute and its hash identifier.
		/// Uses a nonce to keep track of identifiers making them unique after attributes deletion.
		fn attribute_and_id(
			identity: &T::AccountId,
			name: &[u8],
		) -> Option<
			AttributedId<
				BlockNumberFor<T>,
				<<T as Config>::Time as MomentTime>::Moment,
				<T as Config>::MaxNameLength,
				<T as Config>::MaxValueLength,
			>,
		> {
			let bounded_name: BoundedVec<u8, T::MaxNameLength> = name.to_vec().try_into().ok()?;

			let nonce = Self::nonce_of((&identity, bounded_name));

			// Used for first time attribute creation
			let lookup_nonce = match nonce {
				0u64 => 0u64,
				_ => nonce - 1u64,
			};

			// Looks up for the existing attribute.
			// Needs to use actual attribute nonce -1.
			let id = (&identity, name, lookup_nonce).using_encoded(blake2_256);

			if <AttributeOf<T>>::contains_key((&identity, &id)) {
				Some((Self::attribute_of((identity, id)), id))
			} else {
				None
			}
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	//   where &<T as frame_system::Config>::AccountId: PartialEq<<T as
	// frame_system::Config>::AccountId>
	{
		/// Creates a new attribute as part of an identity.
		#[pallet::call_index(0)]
		// #[pallet::weight(T::WeightInfo::add_attribute())]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn add_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
			value: Vec<u8>,
			valid_for: Option<BlockNumberFor<T>>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeNameExceedMax64);

			Self::create_attribute(&who, &identity, &name, &value, valid_for)?;
			// Emit an event that a new id has been added.
			Self::deposit_event(Event::AttributeAdded { identity, name, value, valid_for });
			Ok(())
		}

		///fetch an attribute of an identity
		#[pallet::call_index(1)]
		// #[pallet::weight(T::WeightInfo::get_attribute())]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn get_attribute(
			origin: OriginFor<T>,
			identity: T::AccountId,
			name: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(name.len() <= 64, Error::<T>::AttributeNameExceedMax64);
			let result = Self::attribute_and_id(&identity, &name);

			match result {
				Some(_) => {
					//emit event on read of an attribute
					//may be considered to suppress it
					Self::deposit_event(Event::AttributeFetched { who, identity, name });
				},
				None => {
					//raise error
					return Err(Error::<T>::AttributeNotFound.into());
				},
			}
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// test the rpc interface with simple call
		pub fn get_a_value(i: u32, j: u32) -> u32 {
			i * i + j // return the value i sqaure + j
		}

		/// read the value of an attribute from the chain
		pub fn read_attribute(
			identity: &T::AccountId,
			name: &[u8],
		) -> Option<
			Attribute<
				BlockNumberFor<T>,
				<<T as Config>::Time as MomentTime>::Moment,
				<T as Config>::MaxNameLength,
				<T as Config>::MaxValueLength,
			>,
		> {
			let bounded_name: BoundedVec<u8, T::MaxNameLength> = name.to_vec().try_into().ok()?;
			let nonce = Self::nonce_of((&identity, bounded_name));

			// Used for first time attribute creation
			let lookup_nonce = match nonce {
				0u64 => 0u64,
				_ => nonce - 1u64,
			};

			// Looks up for the existing attribute.
			// Needs to use actual attribute nonce -1.
			let id = (&identity, name, lookup_nonce).using_encoded(blake2_256);

			if <AttributeOf<T>>::contains_key((&identity, &id)) {
				Some(Self::attribute_of((identity, id)))
			} else {
				None
			}
		}
	}

	pub trait DidProvider {
		type AccountId;

		fn is_account_created(identity: &Self::AccountId) -> bool;
		fn creat_new_did(identity: &Self::AccountId) -> Result<(), ()>;
		fn invalidate_did(identity: &Self::AccountId) -> Result<(), ()>;
	}

	impl<T: Config> DidProvider for Pallet<T> {
		type AccountId = T::AccountId;

		//checks if the identity is created already
		fn is_account_created(identity: &Self::AccountId) -> bool {
			let name: Vec<u8> = "VERIFIED".as_bytes().to_vec();
			if Self::attribute_and_id(identity, &name).is_some() {
				return true;
			}
			false
		}
		fn creat_new_did(identity: &Self::AccountId) -> Result<(), ()> {
			let name: Vec<u8> = "VERIFIED".as_bytes().to_vec();
			let value: Vec<u8> = "YES".as_bytes().to_vec();
			let valid_for: Option<BlockNumberFor<T>> = None;

			let result = Self::create_attribute(&identity, &identity, &name, &value, valid_for);
			if let Err(_e) = result {
				// sp_io::logging::log::Error!("error in creating DID VERIFIED attribute:{e:?}");
				// sp_io::logging::log(sp_core::LogLevel::Error, "runtime::io", b"error in creating DID VERIFIED attribute:{e:?}");
				return Err(());
			}
			Ok(())
		}

		fn invalidate_did(identity: &Self::AccountId) -> Result<(), ()> {
			let name: Vec<u8> = "VERIFIED".as_bytes().to_vec();

			let result = Self::reset_attribute(&identity, &identity, &name);
			if let Err(_e) = result {
				// sp_io::logging::log::log!("error in creating DID VERIFIED attribute:{e:?}");
				// sp_io::logging::log(sp_core::LogLevel::Error, "runtime::io", b"error in creating DID VERIFIED attribute:{e:?}");
				return Err(());
			}
			Ok(())
		}
	}
}
