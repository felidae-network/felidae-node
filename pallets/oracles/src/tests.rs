use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_register_oracle() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100_u128 * 10_u128.pow(12);

		// Register an oracle
		assert_ok!(Oracles::register_oracle(RuntimeOrigin::signed(account), deposit));

		// assert the oracle exists
		assert!(Oracles::oracles(account).is_some())
	});
}

#[test]
fn correct_error_on_re_register_oracle() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 100_u128 * 10_u128.pow(12);

		// Register an oracle
		assert_ok!(Oracles::register_oracle(RuntimeOrigin::signed(account), deposit));

		// error OracleAlreadyRegistered upon attempt to re-register
		assert_noop!(
			Oracles::register_oracle(RuntimeOrigin::signed(account), deposit),
			Error::<Test>::OracleAlreadyRegistered
		);
	});
}

#[test]
fn correct_error_on_zero_deposit_register_oracle() {
	new_test_ext().execute_with(|| {
		let account = account_key("//Alice");
		let deposit = 0u128;

		// error InvalidDepositeAmount upon attempt to re-register with zero deposit
		assert_noop!(
			Oracles::register_oracle(RuntimeOrigin::signed(account), deposit),
			Error::<Test>::InvalidDepositeAmount
		);
	});
}

#[test]
fn test_update_protocol_parameters() {
	new_test_ext().execute_with(|| {
		let alice = account_key("//Alice");
		let new_parameters = OracleProtocolParameterValues {
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
		};
		assert_ok!(Oracles::update_protocol_parameters(
			RuntimeOrigin::signed(alice),
			new_parameters.clone()
		));
		let stored_parameters = Oracles::protocol_parameters();
		assert_eq!(stored_parameters, new_parameters);
	});
}
