use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

pub mod mock_data {
	pub const PARTNER_ID: &[u8; 10] = b"pid7hu4210";
	pub const REFERRER_PARTNER_ID: &[u8; 10] = b"rid7hu7h4k";
	pub const PARTNER_INFO: &[u8] = b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
	pub const EVENT_TYPE_ID: &[u8; 10] = b"etid7hu764";
	pub const EVENT_DETAILS: &[u8] = b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
	pub const EVENT_ID: &[u8; 10] = b"eid7hu7890";
	pub const EVENT_CREATION_DETAILS: &[u8] =
		b"bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
	pub const EVENT_STATE: u32 = 1;
	pub const EVENT_VALUE: u128 = 1_000_000_000_000;
}

#[test]
fn partner_registration() {
	new_test_ext().execute_with(|| {
		let account = mock_account("//Alice");

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		// Test for duplicate entry
		assert_noop!(
			AdoptionModule::partner_registration(
				RuntimeOrigin::signed(account),
				mock_data::PARTNER_ID.to_vec(),
				mock_data::PARTNER_INFO.to_vec()
			),
			Error::<Test>::AlreadyAdded
		);
	});
}

#[test]
fn adoption_event_types_creation() {
	new_test_ext().execute_with(|| {
		let alice = mock_account("//Alice");
		let bob = mock_account("//Bob");

		assert_ok!(AdoptionModule::adoption_event_types_creation(
			RuntimeOrigin::signed(alice),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_types_creation(
			RuntimeOrigin::signed(bob),
			b"eid7hu77".to_vec(),
			mock_data::EVENT_DETAILS.to_vec()
		));

		// Test for duplicate entry
		assert_noop!(
			AdoptionModule::adoption_event_types_creation(
				RuntimeOrigin::signed(bob),
				mock_data::EVENT_TYPE_ID.to_vec(),
				mock_data::EVENT_DETAILS.to_vec()
			),
			Error::<Test>::AlreadyAdded
		);
	});
}

#[test]
fn adoption_event_creation() {
	new_test_ext().execute_with(|| {
		let account = mock_account("//Alice");

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_types_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_VALUE,
			account,
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::EVENT_CREATION_DETAILS.to_vec(),
		));

		// Test for duplicate entry
		assert_noop!(
			AdoptionModule::adoption_event_creation(
				RuntimeOrigin::signed(account),
				mock_data::EVENT_ID.to_vec(),
				mock_data::PARTNER_ID.to_vec(),
				mock_data::EVENT_TYPE_ID.to_vec(),
				mock_data::EVENT_VALUE,
				account,
				mock_data::REFERRER_PARTNER_ID.to_vec(),
				mock_data::EVENT_CREATION_DETAILS.to_vec(),
			),
			Error::<Test>::AlreadyAdded
		);
	});
}

#[test]
fn adoption_event_participant_addition() {
	new_test_ext().execute_with(|| {
		let account = mock_account("//Test");
		let alice = mock_account("//Alice");
		let bob = mock_account("//Bob");

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_types_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_VALUE,
			account,
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::EVENT_CREATION_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::event_set_state(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::EVENT_STATE
		));

		assert_ok!(AdoptionModule::adoption_event_participant_addition(
			RuntimeOrigin::signed(account),
			alice,
			mock_data::EVENT_ID.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_participant_addition(
			RuntimeOrigin::signed(account),
			bob,
			mock_data::EVENT_ID.to_vec()
		));

		// Test for duplicate entry
		assert_noop!(
			AdoptionModule::adoption_event_participant_addition(
				RuntimeOrigin::signed(account),
				bob,
				mock_data::EVENT_ID.to_vec()
			),
			Error::<Test>::ParticipantAlreadyAdded
		);
	});
}

#[test]
fn mint_event_participants() {
	new_test_ext().execute_with(|| {
		let account = mock_account("//Test");
		let alice = mock_account("//Alice");
		let bob = mock_account("//Bob");

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(alice),
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_types_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_VALUE,
			account,
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::EVENT_CREATION_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::event_set_state(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::EVENT_STATE
		));

		assert_ok!(AdoptionModule::adoption_event_participant_addition(
			RuntimeOrigin::signed(account),
			alice,
			mock_data::EVENT_ID.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_participant_addition(
			RuntimeOrigin::signed(account),
			bob,
			mock_data::EVENT_ID.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_participant_addition(
			RuntimeOrigin::signed(account),
			account,
			mock_data::EVENT_ID.to_vec()
		));

		assert_ok!(AdoptionModule::mint_event_participants(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec()
		));
	});
}

#[test]
fn event_set_state() {
	new_test_ext().execute_with(|| {
		let account = mock_account("//Test");

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::partner_registration(
			RuntimeOrigin::signed(account),
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::PARTNER_INFO.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_types_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::adoption_event_creation(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::PARTNER_ID.to_vec(),
			mock_data::EVENT_TYPE_ID.to_vec(),
			mock_data::EVENT_VALUE,
			account,
			mock_data::REFERRER_PARTNER_ID.to_vec(),
			mock_data::EVENT_CREATION_DETAILS.to_vec()
		));

		assert_ok!(AdoptionModule::event_set_state(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			mock_data::EVENT_STATE
		));

		assert_ok!(AdoptionModule::event_set_state(
			RuntimeOrigin::signed(account),
			mock_data::EVENT_ID.to_vec(),
			3
		));

		assert_noop!(
			AdoptionModule::event_set_state(
				RuntimeOrigin::signed(account),
				b"idabcxyz".to_vec(),
				mock_data::EVENT_STATE
			),
			Error::<Test>::AdoptionEventDoesNotExist
		);

		assert_noop!(
			AdoptionModule::event_set_state(
				RuntimeOrigin::signed(account),
				mock_data::EVENT_ID.to_vec(),
				4
			),
			Error::<Test>::InvalidEventStateValue
		);
	});
}
