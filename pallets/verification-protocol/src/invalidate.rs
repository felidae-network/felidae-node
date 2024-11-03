use crate::{
	types::{EvalVpResult, IdDocument, IdType, VerificationResult, VerificationStages},
	Config, Error, Pallet, VerificationResults,
};
use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, ensure};
use pallet_did::DidProvider;
use scale_info::TypeInfo;

// Reasons to invalidate an Id
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, Ord, Eq, PartialOrd, Debug)]
pub enum ReasonToInvalidate {
	// a whitelisted ID Document type removed
	// which will entail invalidating all verifications
	// done with that ID Document
	IdDocumentRemoved,
}

pub trait Invalidate<T: Config> {
	fn is_verified(id: &T::AccountId) -> DispatchResult {
		let verification_data = VerificationResults::<T>::get(id);
		// check if current state is valid
		ensure!(
			matches!(
				verification_data,
				Some(VerificationResult { stage: VerificationStages::Done, .. })
			),
			Error::<T>::WrongIdState
		);
		let id_type = match verification_data {
			Some(VerificationResult {
				result: EvalVpResult::Accepted(consumer_details), ..
			}) => IdType::<T>::build(
				consumer_details.type_of_id.into(),
				consumer_details.id_issuing_authority.into(),
				consumer_details.country.into(),
			)?,
			_ => unreachable!(),
		};
		//check if IdType not whitelisted anymore
		ensure!(Pallet::<T>::validate_id_type(&id_type).is_err(), Error::<T>::IdTypeWhitelisted);
		// check status in DID module. it should be ok
		ensure!(<<T as Config>::DidProvider>::is_account_created(&id), Error::<T>::DidNotValid);

		Ok(())
	}
	fn invalidate(&self, id: &T::AccountId) -> DispatchResult;
}

impl<T: Config> Invalidate<T> for ReasonToInvalidate {
	fn invalidate(&self, id: &T::AccountId) -> DispatchResult {
		match self {
			ReasonToInvalidate::IdDocumentRemoved => {
				// Invalidate the verification state
				VerificationResults::<T>::mutate(id, |data| {
					if let Some(d) = data {
						d.stage = VerificationStages::Invalidated;
					}
				});
				let _ = <<T as Config>::DidProvider>::invalidate_did(id);
				// TODO! catch error
			},
			_ => unreachable!(),
		}
		Ok(())
	}
}
