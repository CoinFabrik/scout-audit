use crate as pallet_example_basic;
use crate::*;
use frame_support::{
    assert_ok, derive_impl,
    traits::{ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use std::ops::Index;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        Example: pallet_example_basic,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type Nonce = u64;
    type Hash = H256;
    type RuntimeCall = RuntimeCall;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

impl Config for Test {
    type MagicNumber = ConstU64<1_000_000_000>;
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Count = ConstU32<128>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = RuntimeGenesisConfig {
        system: Default::default(),
        balances: Default::default(),
    }
    .build_storage()
    .unwrap();
    t.into()
}

#[test]
fn basic_test() {
    new_test_ext().execute_with(|| {
        assert!(Dummy::<Test>::get().is_none());
        assert!(Sum::<Test>::get().is_none());

        assert_ok!(Example::insert_dummy(RuntimeOrigin::signed(1), 42_u32));

        let state = Dummy::<Test>::get();
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state.len(), 1);
        assert_eq!(*state.index(0), 42);

        assert!(Sum::<Test>::get().is_none());

        assert_ok!(Example::set_sum(RuntimeOrigin::signed(1)));

        let state = Sum::<Test>::get();
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state, 42);

        assert_ok!(Example::insert_dummy(RuntimeOrigin::signed(1), 37_u32));

        let state = Dummy::<Test>::get();
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state.len(), 2);
        assert_eq!(*state.index(0), 42);
        assert_eq!(*state.index(1), 37);

        let state = Sum::<Test>::get();
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state, 42);

        assert_ok!(Example::set_sum(RuntimeOrigin::signed(1)));

        let state = Sum::<Test>::get();
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state, 42 + 37);
    });
}
