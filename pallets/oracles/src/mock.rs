use crate as pallet_oracles;
use frame_support::{
	derive_impl, parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
	PalletId,
};
use pallet_balances as balances;
use pallet_balances::AccountData;
use sp_core::{sr25519, Pair, H256};
pub use sp_runtime::FixedI64;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

/// Balance of an account.
pub type Balance = u128;

type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) use pallet_oracles::types::ProtocolParameterValues as OracleProtocolParameterValues;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Oracles: pallet_oracles,
	}
);

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
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

// Parameters of oracle pallet
parameter_types! {
	pub const MaxEligibleOracles: u32 = 1000;
	pub const TreasuryPalletId2: PalletId = PalletId(*b"0000vrfr");
}

// Configure the oracle pallet
impl pallet_oracles::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type PalletId = TreasuryPalletId2;
	type MaxEligibleOracles = MaxEligibleOracles;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	balances::GenesisConfig::<Test> {
		balances: vec![
			(account_key("//Alice"), 1000_000_000_000_000),
			(account_key("//Bob"), 1000_000_000_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}

pub fn account_key(s: &str) -> sr25519::Public {
	sr25519::Pair::from_string(&format!("//{}", s), None)
		.expect("static values are valid; qed")
		.public()
}
