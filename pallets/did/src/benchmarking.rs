//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as DID;
#[allow(unused)]
use crate::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

/// Assert that the last event equals the provided one.
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const IDENTITY_STR: &str = "Alice";
// const CALLER_ACCOUNT_STR: &str = "Bob";
const NAME_BYTES: &[u8; 2] = b"id";
const ATTRITUBE_BYTES: &[u8; 17] = b"did:fn:1234567890";

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn add_attribute() {
		let identity: T::AccountId = account(IDENTITY_STR, 0, 0);
		let name_bytes = NAME_BYTES.to_vec();
		let attribute_bytes = ATTRITUBE_BYTES.to_vec();

		#[extrinsic_call]
		add_attribute(
			RawOrigin::Signed(identity.clone()),
			identity.clone(),
			name_bytes.clone(),
			attribute_bytes.clone(),
			None,
		);

		assert_last_event::<T>(
			Event::<T>::AttributeAdded {
				identity: identity.clone(),
				name: name_bytes.clone(),
				value: attribute_bytes.clone(),
				valid_for: None,
			}
			.into(),
		);
	}

	impl_benchmark_test_suite!(DID, crate::mock::new_test_ext(), crate::mock::Test);
}
