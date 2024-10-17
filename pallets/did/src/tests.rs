use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn add_attribute() {
	new_test_ext().execute_with(|| {
		//it should succeed if the attribute is not already present for the DID
		let alice = account_key("Alice");
		let name = b"id";
		let attribute = b"did:fn:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(alice),
			alice,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test for duplicate entry
		assert_noop!(
			DidModule::add_attribute(
				RuntimeOrigin::signed(alice),
				alice,
				name.to_vec(),
				attribute.to_vec(),
				None
			),
			Error::<Test>::DuplicateNotNeeded
		);
	});
}

#[test]
fn read_attribute_by_non_owner() {
	new_test_ext().execute_with(|| {
		let alice = account_key("Alice");
		let bob = account_key("Bob");
		let name = b"id";
		let attribute = b"did:fn:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(alice),
			alice,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test read existing attribute
		assert_ok!(DidModule::get_attribute(RuntimeOrigin::signed(bob), alice, name.to_vec()));
	});
}

#[test]
fn read_attribute_by_owner() {
	new_test_ext().execute_with(|| {
		let alice = account_key("Alice");
		let name = b"id";
		let attribute = b"did:fn:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(alice),
			alice,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		// Test read existing attribute
		assert_ok!(DidModule::get_attribute(RuntimeOrigin::signed(alice), alice, name.to_vec()));

		// Test read non-existing attribute
		assert_noop!(
			DidModule::get_attribute(
				RuntimeOrigin::signed(alice),
				account_key("invalid"),
				name.to_vec()
			),
			Error::<Test>::AttributeNotFound
		);
	});
}

#[test]
fn unauthorized_addition() {
	new_test_ext().execute_with(|| {
		// test for adding attribute of did by unauthorized user
		let alice = account_key("Alice");
		let bob = account_key("Bob");
		let name = b"id";
		let attribute = b"did:fn:1234567890";

		assert_ok!(DidModule::add_attribute(
			RuntimeOrigin::signed(alice),
			alice,
			name.to_vec(),
			attribute.to_vec(),
			None
		));

		assert_noop!(
			DidModule::add_attribute(
				RuntimeOrigin::signed(bob),
				alice,
				name.to_vec(),
				attribute.to_vec(),
				None
			),
			Error::<Test>::NotOwner
		);
	});
}
