use codec::{Decode, Encode};

use scale_info::TypeInfo;

use scale_info::prelude::vec::Vec;
use sp_core::MaxEncodedLen;
use sp_runtime::{
	traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul},
	FixedI64, FixedU128,
};

#[derive(Clone, Debug)]
pub enum Increment {
	Accepted(u8),
	Unaccepted(u8),
	Incomplete(u8),
}

#[derive(Debug, Clone)]
pub struct OracleUpdateData {
	// account_id: A,
	pub incentive_factor: FixedU128,
	pub increment: Increment,
}

/// Oracle agents struct
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Oracle<AccountId, BlockNumber, Balance> {
	pub account_id: AccountId,
	pub balance: Balance,
	pub selection_score: u128,
	pub state: OracleState,
	pub count_of_accepted_submissions: u32,
	pub count_of_un_accepted_submissions: u32,
	pub count_of_incompleted_processes: u32,
	// The time when accuracy score went bellow threshold
	pub threshold_breach_at: Option<BlockNumber>,
	pub reputation_score: FixedI64,
}

// enum to hold the state of the oracles
#[derive(Default, Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum OracleState {
	Active,
	InActive,
	Submitted,
	#[default]
	Pending,
	Deactivated,
	Other,
}

impl<AccountId, BlockNumber, Balance> Oracle<AccountId, BlockNumber, Balance> {
	pub fn is_active(&self) -> bool {
		matches!(self.state, OracleState::Active)
	}

	pub fn accuracy(&self) -> FixedI64 {
		// return 100% for the initial case where all the counts are zero.
		if self.count_of_accepted_submissions +
			self.count_of_un_accepted_submissions +
			self.count_of_incompleted_processes ==
			0
		{
			return FixedI64::from_u32(100)
		}
		let count_of_accepted_submissions = FixedI64::from_u32(self.count_of_accepted_submissions);
		let count_of_un_accepted_submissions =
			FixedI64::from_u32(self.count_of_un_accepted_submissions);
		let count_of_incompleted_processes =
			FixedI64::from_u32(self.count_of_incompleted_processes);
		// let nominator = count_of_accepted_submissions
		// 	.checked_sub(&count_of_un_accepted_submissions)
		// 	.unwrap_or_else(|| FixedI64::min_value());

		let denominator = count_of_accepted_submissions
			.checked_add(&count_of_un_accepted_submissions)
			.unwrap_or_else(|| FixedI64::max_value())
			.checked_add(&count_of_incompleted_processes)
			.unwrap_or_else(|| FixedI64::max_value());
		// result = x/(x+y) * 100
		let result = count_of_accepted_submissions
			.checked_div(&denominator)
			.unwrap_or_else(|| FixedI64::min_value())
			.checked_mul(&100i64.into())
			.unwrap_or_else(|| FixedI64::min_value());

		result
	}

	// Evaluate selection score of a oracle by taking into account the accuracy and reputation
	// score
	pub fn selection_score(self) -> FixedI64 {
		//update accuracy score
		let accuracy = self.accuracy();
		let selection_score = accuracy + self.reputation_score;
		//TODO implement other weights
		selection_score
	}
}

pub trait OracleOperations<AccountId, BlockNumber, Balance> {
	fn sort_on_selection_score(
		list_of_oracles: Vec<Oracle<AccountId, BlockNumber, Balance>>,
	) -> Vec<Oracle<AccountId, BlockNumber, Balance>> {
		list_of_oracles
	}
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub minimum_deposit_for_being_active: u128,
	pub threshold_accuracy_score: FixedI64,
	pub penalty_waiver_score: FixedI64,
	pub resumption_waiting_period: u32,
	pub reward_amount: u128,
	pub penalty_amount: u128,
	pub penalty_amount_not_completed: u128,
	pub accuracy_weight: FixedI64,
	pub reputation_score_weight: FixedI64,
}

impl Default for ProtocolParameterValues {
	fn default() -> Self {
		ProtocolParameterValues {
			minimum_deposit_for_being_active: 100_000_000_000_000,
			threshold_accuracy_score: FixedI64::from_inner(85),
			penalty_waiver_score: FixedI64::from_inner(95),
			// resumption period in number of blocks
			resumption_waiting_period: 100,
			reward_amount: 10_000_000_000_000,
			penalty_amount: 10_000_000_000_000,
			penalty_amount_not_completed: 15_000_000_000_000,
			accuracy_weight: FixedI64::from_inner(1),
			reputation_score_weight: FixedI64::from_inner(1),
		}
	}
}
