// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::types::*;
	use frame_support::{
		// inherent::Vec,
		pallet_prelude::*,
		sp_runtime::traits::AccountIdConversion,
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};

	use frame_support::sp_runtime::SaturatedConversion;
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{Bounded, CheckedAdd, CheckedMul, CheckedSub, Zero},
		ArithmeticError, FixedI64, FixedU128,
	};

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
