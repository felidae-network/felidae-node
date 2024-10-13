//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as AdoptionEvent;
#[allow(unused)]
use crate::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
// use frame_support::assert_noop;

/// Assert that the last event equals the provided one.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[benchmarks]
mod benchmarks {
	use super::*;
	use frame_benchmarking::whitelisted_caller;

	#[benchmark]
	fn partner_registration() {
		let caller: T::AccountId = whitelisted_caller();
		let partner_id = b"pid7hu4210".to_vec();
		let partner_info_cid =
			b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_vec();

		// Calling the extrinsic function
		#[extrinsic_call]
		partner_registration(
			RawOrigin::Signed(caller.clone()),
			partner_id.clone(),
			partner_info_cid.clone(),
		);

		// Assert that the event is emitted correctly
		assert_last_event::<T>(
			Event::<T>::NewPartnerAdded {
				partner_id: partner_id.clone(),
				partner_info_cid: partner_info_cid.clone(),
			}
			.into(),
		);
	}

	impl_benchmark_test_suite!(AdoptionEvent, crate::mock::new_test_ext(), crate::mock::Test);
}
