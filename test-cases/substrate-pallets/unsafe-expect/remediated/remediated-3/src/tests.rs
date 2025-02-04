use crate as pallet_example_basic;
use crate::*;
use frame_support::{assert_ok, derive_impl, dispatch::GetDispatchInfo};
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
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = RuntimeGenesisConfig {
        system: Default::default(),
        balances: Default::default(),
        example: pallet_example_basic::GenesisConfig {
            initial_value: Some(0),
            _phantom: Default::default(),
        },
    }
    .build_storage()
    .unwrap();
    t.into()
}

#[test]
fn weights_work() {
    let call = pallet_example_basic::Call::<Test>::unsafe_get_storage {};
    let info = call.get_dispatch_info();
    assert!(info.weight.ref_time() > 0);
    assert_eq!(
        info.weight,
        <Test as Config>::WeightInfo::unsafe_get_storage()
    );
}

#[test]
fn unsafe_get_storage_works() {
    new_test_ext().execute_with(|| {
        ExampleStorage::<Test>::put(42u32);
        assert_ok!(Example::unsafe_get_storage(RuntimeOrigin::signed(1)));
    });
}

#[test]
fn set_storage_works() {
    new_test_ext().execute_with(|| {
        let test_val = 133u32;
        assert_ok!(Example::set_storage(RuntimeOrigin::signed(1), test_val));
        assert_eq!(ExampleStorage::<Test>::get(), Some(test_val));
    });
}

#[test]
fn clear_storage_works() {
    new_test_ext().execute_with(|| {
        ExampleStorage::<Test>::put(42u32);
        assert_ok!(Example::clear_storage(RuntimeOrigin::signed(1)));
        assert_eq!(ExampleStorage::<Test>::get(), None);
    });
}
