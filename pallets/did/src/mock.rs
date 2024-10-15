use crate as pallet_did;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU16, ConstU64},
};
use scale_info::TypeInfo;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Timestamp: pallet_timestamp,
		DidModule: pallet_did,
	}
);

parameter_types! {
	///max length of id in adoption pallet
	// pub const MaxNameLength: u32 = 256;
	// pub const MaxValueLength: u32 = 256;
	pub const MinimumPeriod1: u64 = 5;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[derive(TypeInfo)]
pub struct MaxNameLength;
impl frame_support::traits::Get<u32> for MaxNameLength {
    fn get() -> u32 {
        256
    }
}
impl Default for MaxNameLength {
    fn default() -> Self {
        MaxNameLength
    }
}

#[derive(TypeInfo)]
pub struct MaxValueLength;
impl frame_support::traits::Get<u32> for MaxValueLength {
    fn get() -> u32 {
        256
    }
}
impl Default for MaxValueLength {
    fn default() -> Self {
        MaxValueLength
    }
}

impl pallet_did::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Time = Timestamp;
	type MaxNameLength = MaxNameLength;
    type MaxValueLength = MaxValueLength;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod1;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

pub fn account_key(s: &str) -> sr25519::Public {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}