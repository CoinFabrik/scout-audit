use crate as pallet_example_basic;
use crate::*;
use frame_support::{
    assert_ok, derive_impl,
    dispatch::{DispatchInfo, GetDispatchInfo},
    traits::{ConstU64, OnInitialize},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

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
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = RuntimeGenesisConfig {
        system: Default::default(),
        balances: Default::default(),
        example: pallet_example_basic::GenesisConfig {
            dummy: 42,
            bar: alloc::vec![(1, 2), (2, 3)],
            foo: 24,
        },
    }
    .build_storage()
    .unwrap();
    t.into()
}

#[test]
fn test_large_value() {
    new_test_ext().execute_with(|| {
        assert!(Example::accumulate_dummy(RuntimeOrigin::signed(1), 1001_u64).is_err());
    });
}

#[test]
fn it_works_for_optional_value() {
    new_test_ext().execute_with(|| {
        let val1 = 42;
        let val2 = 27;
        assert_eq!(Dummy::<Test>::get(), Some(val1));

        assert_ok!(Example::accumulate_dummy(RuntimeOrigin::signed(1), val2));
        assert_eq!(Dummy::<Test>::get(), Some(val1 + val2));

        <Example as OnInitialize<u64>>::on_initialize(2);
        assert_ok!(Example::accumulate_dummy(RuntimeOrigin::signed(1), val1));
        assert_eq!(Dummy::<Test>::get(), Some(val1 + val2 + val1));
    });
}

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        assert_eq!(Foo::<Test>::get(), 24);
        assert_ok!(Example::accumulate_foo(RuntimeOrigin::signed(1), 1));
        assert_eq!(Foo::<Test>::get(), 25);
    });
}

#[test]
fn set_dummy_works() {
    new_test_ext().execute_with(|| {
        let test_val = 133;
        assert_ok!(Example::set_dummy(RuntimeOrigin::root(), test_val));
        assert_eq!(Dummy::<Test>::get(), Some(test_val));
    });
}

#[test]
fn signed_ext_watch_dummy_works() {
    new_test_ext().execute_with(|| {
        let call = pallet_example_basic::Call::set_dummy { new_value: 10 }.into();
        let info = DispatchInfo::default();

        assert_eq!(
            WatchDummy::<Test>(PhantomData)
                .validate(&1, &call, &info, 150)
                .unwrap()
                .priority,
            u64::MAX,
        );
        assert_eq!(
            WatchDummy::<Test>(PhantomData).validate(&1, &call, &info, 250),
            InvalidTransaction::ExhaustsResources.into(),
        );
    })
}

#[test]
fn counted_map_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(CountedMap::<Test>::count(), 0);
        CountedMap::<Test>::insert(3, 3);
        assert_eq!(CountedMap::<Test>::count(), 1);
    })
}

#[test]
fn weights_work() {
    let default_call = pallet_example_basic::Call::<Test>::accumulate_dummy { increase_by: 10 };
    let info1 = default_call.get_dispatch_info();
    assert!(info1.weight.ref_time() > 0);
    assert_eq!(
        info1.weight,
        <Test as Config>::WeightInfo::accumulate_dummy()
    );

    let custom_call = pallet_example_basic::Call::<Test>::set_dummy { new_value: 20 };
    let info2 = custom_call.get_dispatch_info();
    assert!(info1.weight.ref_time() > info2.weight.ref_time());
}
