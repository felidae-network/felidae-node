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
// pub mod weights;
// pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::types::*;
	use frame_support::{
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

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	pub struct UpdateData {
		pub incr_accepted_submissions: Option<u32>,
		pub incr_un_accepted_submissions: Option<u32>,
		pub incr_incompleted_processes: Option<u32>,
	}

	/// The pallet's configuration trait.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Currency handler for the pallet.
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type MaxEligibleOracles: Get<u32>;
	}

	// Storage to hold the list of Oracles
	#[pallet::storage]
	#[pallet::getter(fn oracles)]
	pub type Oracles<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Oracle<T::AccountId, BlockNumberFor<T>, BalanceOf<T>>,
		OptionQuery,
	>;

	// Storage to hold the list of active eligible oracles who is ready to take task
	#[pallet::storage]
	#[pallet::getter(fn eligible_oracles)]
	pub type EligibleOracles<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxEligibleOracles>, ValueQuery>;

	// Store the protocol parameters
	#[pallet::storage]
	#[pallet::getter(fn protocol_parameters)]
	pub type ProtocolParameters<T> = StorageValue<_, ProtocolParameterValues, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// parameters. [oracle_account_id]
		OracleRegistrationRequest { oracle: T::AccountId },
		/// parameters. [oracle_account_id, amount]
		OracleDeposite { oracle: T::AccountId, amount: BalanceOf<T> },
		/// Update protocol parameters for stages
		ParametersUpdated {
			minimum_deposit_for_being_active: u128,
			threshold_accuracy_score: FixedI64,
			penalty_waiver_score: FixedI64,
			resumption_waiting_period: u32,
			reward_amount: u128,
			penalty_amount: u128,
			penalty_amount_not_completed: u128,
			accuracy_weight: FixedI64,
			reputation_score_weight: FixedI64,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Oracle already registered
		OracleAlreadyRegistered,
		/// Oracle not registered
		OracleNotRegistered,
		InvalidDepositeAmount,
		/// Erron in pallet_account_id  generation
		PalletAccountIdFailure,
		/// Error in updating value
		ArithmeticOverflow,
	}

	struct RandomHexNumber {
		index: usize,
		nums: Vec<u8>,
	}
	impl Iterator for RandomHexNumber {
		type Item = u8;
		fn next(&mut self) -> Option<Self::Item> {
			let random_hex = self.nums[self.index % self.nums.len()];
			self.index += 1;
			Some(random_hex)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// At block finalization
		fn on_finalize(_now: BlockNumberFor<T>) {
			let last_oracles = Self::eligible_oracles().to_vec();
			let mut oracles: Vec<Oracle<T::AccountId, BlockNumberFor<T>, BalanceOf<T>>> =
				Oracles::<T>::iter_values()
					.filter(|v| v.state == OracleState::Active)
					.map(|v| v)
					.collect();
			let parent_blockhash = <frame_system::Pallet<T>>::parent_hash();

			let decimal_values: Vec<u8> = parent_blockhash
				.as_ref()
				.iter()
				.flat_map(|byte| {
					let high_nibble: u8 = (byte >> 4) & 0xF;
					let low_nibble: u8 = byte & 0xF;
					Vec::from([high_nibble, low_nibble])
				})
				.collect();
			let mut rand_number = RandomHexNumber { index: 0, nums: decimal_values };
			let mut i = 0;
			// sort by accuracy of the oracles
			oracles.sort_by_cached_key(|k| {
				//For one out of 16 probable cases randomize the score by multiplying a random
				// number between 0 to 1.5
				let accuracy = if rand_number
					.next()
					.expect("random number generator should never run out !!!") ==
					15
				{
					k.accuracy() *
						FixedI64::from_u32(
							rand_number
								.next()
								.expect("random number generator  should never run out !!!") as u32,
						) / 10.into()
				} else {
					k.accuracy()
				};
				i += 1;
				sp_std::cmp::Reverse(accuracy)
			});

			let new_oracles: Vec<T::AccountId> =
				oracles.iter().map(|v| v.account_id.clone()).collect();

			if last_oracles != new_oracles {
				if new_oracles.len() as u32 > T::MaxEligibleOracles::get() {
					log::warn!(
						"next oracles list larger than {}, truncating",
						T::MaxEligibleOracles::get(),
					);
				}
				let bounded = <BoundedVec<_, T::MaxEligibleOracles>>::truncate_from(new_oracles);

				EligibleOracles::<T>::put(bounded);
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register an oracle. Takes following parameters
		/// 1. deposit amount
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(0)]
		pub fn register_oracle(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// check if the oracle is already registered
			ensure!(!<Oracles<T>>::contains_key(who.clone()), Error::<T>::OracleAlreadyRegistered);
			// Check that the deposited value is greater than zero.
			ensure!(deposit > Zero::zero(), Error::<T>::InvalidDepositeAmount);

			let pallet_account: T::AccountId = Self::pallet_account_id()?;
			T::Currency::transfer(&who, &pallet_account, deposit, ExistenceRequirement::KeepAlive)?;
			let minimum_deposit_for_being_active: BalanceOf<T> = Self::protocol_parameters()
				.minimum_deposit_for_being_active
				.saturated_into::<BalanceOf<T>>();

			let state = if deposit >= minimum_deposit_for_being_active {
				OracleState::Active
			} else {
				OracleState::Pending
			};
			let oracle = Oracle {
				account_id: who.clone(),
				balance: deposit,
				selection_score: 0,
				state,
				count_of_accepted_submissions: 0u32,
				count_of_un_accepted_submissions: 0u32,
				count_of_incompleted_processes: 0u32,
				threshold_breach_at: None.into(),
				reputation_score: sp_runtime::FixedI64::min_value(),
			};

			// Update Oracles storage.
			<Oracles<T>>::insert(who.clone(), oracle.clone());

			// Emit an event.
			Self::deposit_event(Event::OracleRegistrationRequest { oracle: who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(1)]
		pub fn oracle_deposit(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// check if the oracle is already registered
			ensure!(<Oracles<T>>::contains_key(who.clone()), Error::<T>::OracleNotRegistered);
			// Check that the deposited value is greater than zero.
			ensure!(deposit > Zero::zero(), Error::<T>::InvalidDepositeAmount);

			let minimum_deposit_for_being_active = ProtocolParameters::<T>::get()
				.minimum_deposit_for_being_active
				.saturated_into::<BalanceOf<T>>();

			// update balance and change state if required
			Oracles::<T>::try_mutate(who.clone(), |v| -> DispatchResult {
				if let Some(ref mut oracle) = v {
					let pallet_account: T::AccountId = Self::pallet_account_id()?;
					T::Currency::transfer(
						&who,
						&pallet_account,
						deposit,
						ExistenceRequirement::KeepAlive,
					)?;

					oracle.balance =
						oracle.balance.checked_add(&deposit).ok_or(ArithmeticError::Overflow)?;
					if oracle.balance >= minimum_deposit_for_being_active {
						oracle.state = OracleState::Active;
					}
				}
				Ok(())
			})?;

			// Emit an event.
			Self::deposit_event(Event::OracleDeposite { oracle: who, amount: deposit });
			// Return a successful DispatchResult
			Ok(())
		}

		/// Change protocol parameters
		/// takes new parameters and updates the default value
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		#[pallet::call_index(2)]
		pub fn update_protocol_parameters(
			origin: OriginFor<T>,
			new_parameters: ProtocolParameterValues,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			ProtocolParameters::<T>::put(&new_parameters);

			Self::deposit_event(Event::ParametersUpdated {
				minimum_deposit_for_being_active: new_parameters.minimum_deposit_for_being_active,
				threshold_accuracy_score: new_parameters.threshold_accuracy_score,
				penalty_waiver_score: new_parameters.penalty_waiver_score,
				resumption_waiting_period: new_parameters.resumption_waiting_period,
				reward_amount: new_parameters.reward_amount,
				penalty_amount: new_parameters.penalty_amount,
				penalty_amount_not_completed: new_parameters.penalty_amount_not_completed,
				accuracy_weight: new_parameters.accuracy_weight,
				reputation_score_weight: new_parameters.reputation_score_weight,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn _sub_account_id(id: T::AccountId) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(id)
		}

		pub(crate) fn pallet_account_id() -> Result<T::AccountId, Error<T>> {
			if let Some(account) = T::PalletId::get().try_into_account() {
				Ok(account)
			} else {
				Err(Error::<T>::PalletAccountIdFailure.into())
			}
		}

		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

	pub trait OraclesProvider {
		type AccountId;
		type UpdateData;
		type BlockNumber;

		fn get_oracles() -> Vec<Self::AccountId>;
		fn update_oracle_profiles(
			data: Vec<(Self::AccountId, Self::UpdateData)>,
			current_block: Self::BlockNumber,
		) -> Result<(), ArithmeticError>;
	}

	impl<T: Config> OraclesProvider for Pallet<T> {
		type AccountId = T::AccountId;
		type UpdateData = OracleUpdateData;
		type BlockNumber = BlockNumberFor<T>;

		fn get_oracles() -> Vec<Self::AccountId> {
			EligibleOracles::<T>::get().to_vec()
		}
		fn update_oracle_profiles(
			data: Vec<(Self::AccountId, Self::UpdateData)>,
			current_block: Self::BlockNumber,
		) -> Result<(), ArithmeticError> {
			let parameters = Self::protocol_parameters();
			for (who, update_data) in data.iter() {
				Oracles::<T>::try_mutate(who, |v| -> Result<(), ArithmeticError> {
					if let Some(ref mut oracle) = v {
						let accuracy = oracle.accuracy();
						match update_data.increment {
							Increment::Accepted(n) => {
								oracle.count_of_accepted_submissions = oracle
									.count_of_accepted_submissions
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;

								// reward only if the accuracy score is equal to or higher than the
								// threshold
								// and
								// the oracle is not serving in the resumption period
								if parameters.threshold_accuracy_score <= accuracy &&
									oracle.threshold_breach_at.is_none()
								{
									// reward*factor + reward
									let amount = FixedU128::from_inner(parameters.reward_amount)
										.checked_mul(&update_data.incentive_factor)
										.ok_or(ArithmeticError::Overflow)?
										.checked_add(&FixedU128::from_inner(
											parameters.reward_amount,
										))
										.ok_or(ArithmeticError::Overflow)?;

									let incentive_amount: BalanceOf<T> =
										amount.into_inner().saturated_into();
									oracle.balance = oracle
										.balance
										.checked_add(&incentive_amount)
										.ok_or(ArithmeticError::Overflow)?;

									let _ = T::Currency::transfer(
										&Self::account_id(),
										&oracle.account_id,
										incentive_amount,
										ExistenceRequirement::KeepAlive,
									);
									// Activate if balance goes above limit and in InActive state
									if oracle.balance >=
										parameters
											.minimum_deposit_for_being_active
											.saturated_into() && oracle.state == OracleState::InActive
									{
										oracle.state = OracleState::Active;
									}
								}
								// check if accuracy goes above the threshold and resumption period
								// is over
								if oracle.accuracy() >= parameters.threshold_accuracy_score {
									if let Some(crossed_down_at) = oracle.threshold_breach_at {
										// check if resumption period is over
										if current_block >
											crossed_down_at +
												parameters.resumption_waiting_period.into()
										{
											// record the breach time
											oracle.threshold_breach_at = None;
										}
									}
								}
							},
							Increment::Unaccepted(n) => {
								oracle.count_of_un_accepted_submissions = oracle
									.count_of_un_accepted_submissions
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;

								// waive penalty if accuracy is higher than the waiver threshold
								if parameters.penalty_waiver_score > accuracy {
									let penalty_amount =
										FixedU128::from_inner(parameters.penalty_amount)
											.checked_add(
												&FixedU128::from_inner(parameters.penalty_amount)
													.checked_mul(&update_data.incentive_factor)
													.ok_or(ArithmeticError::Overflow)?,
											)
											.ok_or(ArithmeticError::Overflow)?;

									let incentive_amount: BalanceOf<T> =
										penalty_amount.into_inner().saturated_into();

									oracle.balance = oracle
										.balance
										.checked_sub(&incentive_amount)
										.ok_or(ArithmeticError::Overflow)?;

									// InActivate if balance goes bellow limit
									if oracle.balance <
										parameters
											.minimum_deposit_for_being_active
											.saturated_into()
									{
										oracle.state = OracleState::InActive;
									}
								}
								// check if accuracy goes bellow the threshold
								if oracle.accuracy() < parameters.threshold_accuracy_score &&
									oracle.threshold_breach_at.is_none()
								{
									// let it remain active as per new thought
									// record the breach time
									oracle.threshold_breach_at = Some(current_block);
								}
							},
							Increment::Incomplete(n) => {
								oracle.count_of_incompleted_processes = oracle
									.count_of_incompleted_processes
									.checked_add(n.into())
									.ok_or(ArithmeticError::Overflow)?;

								// waive penalty if accuracy is higher than the waiver threshold
								if parameters.penalty_waiver_score > accuracy {
									let penalty_amount = FixedU128::from_inner(
										parameters.penalty_amount_not_completed,
									)
									.checked_add(
										&FixedU128::from_inner(
											parameters.penalty_amount_not_completed,
										)
										.checked_mul(&update_data.incentive_factor)
										.ok_or(ArithmeticError::Overflow)?,
									)
									.ok_or(ArithmeticError::Overflow)?;

									let incentive_amount: BalanceOf<T> =
										penalty_amount.into_inner().saturated_into();

									oracle.balance = oracle
										.balance
										.checked_sub(&incentive_amount)
										.ok_or(ArithmeticError::Overflow)?;

									if oracle.balance <
										parameters
											.minimum_deposit_for_being_active
											.saturated_into()
									{
										oracle.state = OracleState::InActive;
									}
								}
								// check if accuracy goes bellow the threshold
								if oracle.accuracy() < parameters.threshold_accuracy_score &&
									oracle.threshold_breach_at.is_none()
								{
									// let it remain active as per new thought
									// record the breach time
									oracle.threshold_breach_at = Some(current_block);
								}
							},
						}
					} else {
						log::error!("+++++++++++++Oracle not found+++++++++++++++++");
					}
					Ok(())
				})?;
			}
			Ok(())
		}
	}
}
