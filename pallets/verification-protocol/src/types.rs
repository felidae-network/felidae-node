use codec::{Decode, Encode, MaxEncodedLen};

use crate::Config;
pub use pallet_oracles::types::{Increment, OracleUpdateData};
use frame_system::pallet_prelude::*;
use frame_support::{pallet_prelude::ConstU32, BoundedVec};

use core::{
	cmp::{Eq, Ordering, PartialEq},
	marker::PhantomData,
};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	traits::{Bounded, CheckedDiv, CheckedMul},
	FixedI64, FixedU128,
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Struct of the did verification result
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct VerificationResult<T: Config> {
	pub consumer_account_id: T::AccountId,
	pub submitted_at: BlockNumberFor<T>,
	pub completed_at: BlockNumberFor<T>,
	pub list_of_documents: BoundedVec<u8, T::MaxLengthListOfDocuments>,
	pub did_creation_status: DidCreationStatus,
	pub result: EvalVpResult,
	pub stage: VerificationStages,
}

impl<T: Config> VerificationResult<T> {
	pub fn from_completed_request(
		completed_request: VerificationRequest<T>,
		result: EvalVpResult,
		did_creation_status: DidCreationStatus,
		completed_at: BlockNumberFor<T>,
	) -> Self {
		Self {
			consumer_account_id: completed_request.consumer_account_id,
			submitted_at: completed_request.submitted_at,
			completed_at,
			list_of_documents: completed_request.list_of_documents,
			did_creation_status,
			result,
			stage: VerificationStages::Done,
		}
	}
}

/// Struct of the did verification request submitted by the consumer
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct VerificationRequest<T: Config> {
	pub consumer_account_id: T::AccountId,
	pub submitted_at: BlockNumberFor<T>,
	pub list_of_documents: BoundedVec<u8, T::MaxLengthListOfDocuments>,
	pub did_creation_status: DidCreationStatus,
	pub state: StateConfig<BlockNumberFor<T>>,
	pub round_number: u8,
}

/// Enumurates the possible Did Creation Statuses
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, Copy, MaxEncodedLen)]
pub enum DidCreationStatus {
	/// Pending on submission
	#[default]
	Pending,
	/// Created at the end if accepted eval process
	Created,
	/// Rejected at the end if rejected by eval process
	Rejected,
	/// Eval process successful but same id hashes exists already
	RejectedDuplicate,
	/// Evaluation of verificaiton parameter Failed
	Failed,
}

/// State parameters of different stages of the request
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, MaxEncodedLen)]
pub struct StateConfig<BlockNumber> {
	pub allot: StateAttributes<BlockNumber>,
	pub ack: StateAttributes<BlockNumber>,
	pub submit_vp: StateAttributes<BlockNumber>,
	pub reveal: StateAttributes<BlockNumber>,
	pub eval_vp_result: Option<EvalVpResult>,
	pub eval_vp_state: Option<EvalVpState>,
	pub stage: VerificationStages,
}

/// Enumerates stages of verification requests
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, MaxEncodedLen)]
pub enum VerificationStages {
	#[default]
	Submitted,
	/// In One or Many stages among Allot, Ack and Submit-verification-parameters stages
	AllotAckVp,
	/// In the reveal stage
	Reveal,
	/// In the Eval Stage
	Eval,
	/// Completed all stages
	Done,
	/// upon revoke or challenge
	Invalidated,
}

/// Attributes of a particular Stage
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, MaxEncodedLen)]
pub struct StateAttributes<BlockNumber> {
	pub done_count_of_verifiers: u16,
	pub pending_count_of_verifiers: u16,
	/// whether this is to be fulfilled or Done
	pub state: bool,
	/// Start time of the state true
	pub started_at: BlockNumber,
	/// Completion time of the fullfilment
	pub ended_at: Option<BlockNumber>,
	/// Duration for which state was True
	pub state_duration: u32,
}

/// Enumerates the results of the Evaluation of veri-para  
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, MaxEncodedLen)]
pub enum EvalVpResult {
	/// Eval process started
	#[default]
	Pending,
	/// Accepted and ll proceed to created DID
	Accepted(ConsumerDetails),
	/// Rejected and DID ll not be created
	Rejected,
	/// Eval could not result into a clear Accept or Reject. Consider it Failed
	CantDecideAkaFailed,
}

/// Enumerates states of evaluation process
#[derive(Eq, PartialEq, PartialOrd, TypeInfo, Ord, Clone, Encode, Decode, Default, Debug, MaxEncodedLen)]
pub enum EvalVpState {
	#[default]
	Pending,
	Done,
}

/// Verifier agents struct
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub struct Verifier<AccountId> {
	pub account_id: AccountId,
	pub score: u32,
	pub state: VerifierState,
	pub count_of_accepted_submissions: u128,
	pub count_of_rejected_submissions: u128,
	pub count_of_incompleted_processes: u128,
}

// enum to hold the state of the verifiers
#[derive(Default, Clone, Encode, Decode, PartialEq, TypeInfo, Debug)]
#[scale_info(skip_type_params(T))]
pub enum VerifierState {
	Active,
	InActive,
	Submitted,
	#[default]
	Pending,
	Deactivated,
	Other,
}

/// verification parameter submitted by the verifier
/// Either reject or Accept with the hash of the verification data
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, Ord, Eq, PartialOrd, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum RevealedParameters {
	Reject,
	Accept(ConsumerDetails),
}

/// Struct to hold the process data of verifier for every verification request
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct VerificationProcessData<T: Config> {
	pub verifier_account_id: T::AccountId,
	/// at blocknumber
	pub allotted_at: Option<BlockNumberFor<T>>,
	/// ack with(blocknumber, confidence_score)
	pub acknowledged: Option<(BlockNumberFor<T>, u8)>,
	pub data: Option<(BlockNumberFor<T>, H256)>,
	pub revealed_data: Option<(BlockNumberFor<T>, RevealedParameters)>,
}

/// allot a task to a particular verifier
impl<T: Config> VerificationProcessData<T> {
	// static konst: u8 = 100u8;

	pub fn allot_to_verifier(verifier: T::AccountId, current_block: BlockNumberFor<T>) -> Self {
		VerificationProcessData {
			verifier_account_id: verifier,
			allotted_at: Some(current_block),
			acknowledged: None.into(),
			data: None.into(),
			revealed_data: None.into(),
		}
	}

	// checks if verifier completed the process and returns
	pub fn with_completed(self) -> Option<VerificationProcessDataItem<T>> {
		if let VerificationProcessData {
			verifier_account_id,
			allotted_at: Some(allotted_at),
			acknowledged: Some(acknowledged),
			data: Some(data),
			revealed_data: Some(revealed_data),
		} = self
		{
			Some(VerificationProcessDataItem::<T> {
				verifier_account_id,
				allotted_at,
				acknowledged,
				data,
				revealed_data,
			})
		} else {
			None
		}
	}
}

#[derive(Eq, PartialEq, Clone)]
pub struct VerificationProcessDataItem<T: Config> {
	pub verifier_account_id: T::AccountId,
	pub allotted_at: BlockNumberFor<T>,
	/// ack with(blocknumber, confidence_score)
	pub acknowledged: (BlockNumberFor<T>, u8),
	pub data: (BlockNumberFor<T>, H256),
	pub revealed_data: (BlockNumberFor<T>, RevealedParameters),
}

impl<T: Config> VerificationProcessDataItem<T> {
	// duration between ack and submitting the verification data
	pub fn timetaken(&self) -> BlockNumberFor<T> {
		let VerificationProcessDataItem { acknowledged, data, .. } = self;
		acknowledged.0 - data.0
	}
	// returns true if the final result is same as its submission
	pub fn is_submission_is_result(&self, s: &EvalVpResult) -> bool {
		let VerificationProcessDataItem { revealed_data: (_, revealed_parameters), .. } = self;
		match s {
			EvalVpResult::Rejected =>
				if *revealed_parameters == RevealedParameters::Reject {
					true
				} else {
					false
				},
			EvalVpResult::Accepted(c) =>
				if *revealed_parameters == RevealedParameters::Accept(c.clone()) {
					true
				} else {
					false
				},
			_ => false, // matches to no one
		}
	}
	// Returns the confidence score set at the time of acknowledgement
	pub fn confidence_score(&self) -> u8 {
		let Self { acknowledged: (_, score), .. } = self;
		*score
	}
}

impl<T: Config> PartialOrd for VerificationProcessDataItem<T> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(&other))
	}
}

impl<T: Config> Ord for VerificationProcessDataItem<T> {
	fn cmp(&self, other: &Self) -> Ordering {
		let Self { acknowledged: (ack_at, _), data: (submit_at, _), .. } = self;

		let Self { acknowledged: (ack_at_other, _), data: (submit_at_other, _), .. } = other;
		(*submit_at - *ack_at).cmp(&(*submit_at_other - *ack_at_other))
	}
}

#[derive(Clone)]
pub enum Incentive<Balance> {
	Reward(Balance),
	Penalty(Balance),
}

impl<T: Config> VerificationProcessData<T> {
	// evaluation the winner submission by majority wins
	// takes the list of the Self
	// returns : result=> submission with clear majority, list of (verifier_account,
	// update_data_of_verifier)
	pub fn eval_incentive(
		submissions: Vec<Self>,
		threshold_winning_percentage: u8,
	) -> (EvalVpResult, Vec<(T::AccountId, OracleUpdateData)>) {
		let mut data: Vec<(T::AccountId, OracleUpdateData)> = Vec::new();
		let mut completed_subs: Vec<VerificationProcessDataItem<T>> = Vec::new();
		submissions.into_iter().for_each(|vpr| {
			let verifier = vpr.verifier_account_id.clone();
			if let Some(rd) = vpr.with_completed() {
				completed_subs.push(rd)
			} else {
				data.push((
					verifier,
					OracleUpdateData {
						increment: Increment::Incomplete(1),
						incentive_factor: FixedU128::from_inner(1u128),
					},
				))
			}
		});

		let (result, partial_incentive_data) =
			VerificationProcessDataItem::eval_incentive_on_completed(
				completed_subs,
				threshold_winning_percentage,
			);
		data.extend(partial_incentive_data);
		(result, data)
	}
}

impl<T: Config> VerificationProcessDataItem<T> {
	// evaluation the winner submission by majority wins
	// takes the list of the Self
	// returns : result=> submission with clear majority
	fn eval_result(
		params: &[RevealedParameters],
		threshold_winning_percentage: u8,
	) -> EvalVpResult {
		let mut counts = BTreeMap::new();
		for p in params {
			let count = counts.entry(p).or_insert(0);
			*count += 1;
		}
		// counts
		let mut max_count = 0;
		let mut max_variant = None;
		let mut max_count_2 = 0;
		let mut max_variant_2 = None;
		for (discr, count) in &counts {
			if *count > max_count {
				max_count = *count;
				max_variant = Some(discr);
			} else if *count > max_count_2 {
				max_count_2 = *count;
				max_variant_2 = Some(discr);
			}
		}

		if max_variant.is_some() {
			if max_variant_2.is_some() {
				if max_count == max_count_2 {
					// there is a tie and no clear majority
					max_variant = None;
				}
			} else {
				let winnig_percentage = FixedI64::from_u32(max_count)
					.checked_div(&FixedI64::from_inner(params.len() as i64))
					.unwrap_or_else(|| FixedI64::min_value())
					.checked_mul(&FixedI64::from_inner(100))
					.unwrap_or_else(|| FixedI64::min_value());
				// check if threshold_winning percent is more than the threshold.
				if winnig_percentage < FixedI64::from_inner(threshold_winning_percentage as i64) {
					max_variant = None;
				}
			}
		}

		let result: EvalVpResult = match max_variant {
			Some(variant) => match variant {
				RevealedParameters::Reject => EvalVpResult::Rejected,
				RevealedParameters::Accept(d) => EvalVpResult::Accepted(d.clone()),
			},
			None => EvalVpResult::CantDecideAkaFailed,
		};
		result
	}

	// evaluation the winner submission by majority wins
	// takes the list of the Self where verification process has been completed by the verifier by
	// revealing the data returns : result=> submission with clear majority, list of
	// (verifier_account, update_data_of_verifier)
	fn eval_incentive_on_completed(
		submissions: Vec<Self>,
		threshold_winning_percentage: u8,
	) -> (EvalVpResult, Vec<(T::AccountId, OracleUpdateData)>) {
		// let mut data: BTreeMap<K, V> = BTreeMap::new();
		let entries: Vec<RevealedParameters> = submissions
			.iter()
			.map(|vpr| {
				let (_, rd) = vpr.revealed_data.clone();
				rd
			})
			.collect();
		let result: EvalVpResult = Self::eval_result(&entries[..], threshold_winning_percentage);
		// let fastest = submissions.iter().min().ok_or(())?;
		// let slowest = submissions.iter().min().ok_or(())?;
		// let denominator = fastest.timetaken() - slowest.timetaken();
		let r = submissions
			.iter()
			.map(|x| {
				let increment = if x.is_submission_is_result(&result) {
					Increment::Accepted(1)
				} else {
					Increment::Unaccepted(1)
				};
				(
					x.verifier_account_id.clone(),
					OracleUpdateData {
						increment,
						incentive_factor: FixedU128::from_rational(
							x.confidence_score() as u128,
							100u128,
						),
					},
				)
			})
			.collect::<Vec<_>>();

		(result, r)
	}
}

/// Struct of consumer personal data
/// hash1 may be hash of (name + DOB + Fathers name)
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, Ord, Eq, PartialOrd, MaxEncodedLen)]
pub struct ConsumerDetails {
	pub country: BoundedVec<u8, ConstU32<100>>,
	pub id_issuing_authority: BoundedVec<u8, ConstU32<150>>,
	pub type_of_id: BoundedVec<u8, ConstU32<150>>,
	pub hash1_name_dob_father: Option<H256>,
	pub hash2_name_dob_mother: Option<H256>,
	pub hash3_name_dob_guardian: Option<H256>,
}

impl ConsumerDetails {
	pub fn hashes(&self) -> Vec<H256> {
		let mut result = Vec::new();
		if let Some(hash) = self.hash1_name_dob_father {
			result.push(hash);
		}
		if let Some(hash) = self.hash2_name_dob_mother {
			result.push(hash);
		}
		if let Some(hash) = self.hash3_name_dob_guardian {
			result.push(hash);
		}
		result
	}
}

/// Struct of verification protocol parameters
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Debug, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ProtocolParameterValues {
	pub max_length_list_of_documents: u16,
	pub min_count_at_allot_stage: u16,
	pub min_count_at_ack_accept_stage: u16,
	pub min_count_at_submit_vp_stage: u16,
	pub min_count_at_reveal_stage: u16,
	pub max_waiting_time_at_stages: u32,
	pub threshold_winning_percentage: u8,
}

impl Default for ProtocolParameterValues {
	fn default() -> Self {
		ProtocolParameterValues {
			max_length_list_of_documents: 150,
			min_count_at_allot_stage: 16,
			min_count_at_ack_accept_stage: 12,
			min_count_at_submit_vp_stage: 8,
			min_count_at_reveal_stage: 4,
			max_waiting_time_at_stages: 1200,
			threshold_winning_percentage: 66,
		}
	}
}

pub type Country = BoundedVec<u8, ConstU32<100>>;
pub type IdIssuer = BoundedVec<u8, ConstU32<150>>;
pub type IdName = BoundedVec<u8, ConstU32<150>>;

pub trait IdDocument {
	type IdType;
	type Error;

	fn build(name: Vec<u8>, issuer: Vec<u8>, country: Vec<u8>)
		-> Result<Self::IdType, Self::Error>;
}

#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Ord, Eq, PartialOrd, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct IdType<T: Config> {
	pub name: IdName,
	pub issuer: IdIssuer,
	pub country: Country,
	_marker: PhantomData<T>,
}
use sp_std::fmt;
impl<T: Config> fmt::Debug for IdType<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"IdType {{ name: {:?}, issuer: {:?}, country: {:?} }}",
			self.name, self.issuer, self.country
		)
	}
}

use crate::Error;
impl<T: Config> IdDocument for IdType<T> {
	type IdType = IdType<T>;
	type Error = Error<T>;

	fn build(
		name: Vec<u8>,
		issuer: Vec<u8>,
		country: Vec<u8>,
	) -> Result<Self::IdType, Self::Error> {
		let bounded_name: IdName = name.try_into().map_err(|_| Error::<T>::InvalidIdName)?;
		let bounded_issuer: IdIssuer =
			issuer.try_into().map_err(|_| Error::<T>::InvalidIdIssuer)?;
		let bounded_country: Country =
			country.try_into().map_err(|_| Error::<T>::InvalidCountry)?;
		Ok(Self {
			name: bounded_name,
			issuer: bounded_issuer,
			country: bounded_country,
			_marker: Default::default(),
		})
	}
}
