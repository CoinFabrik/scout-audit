#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    dispatch::{ClassifyDispatch, DispatchClass, DispatchResult, Pays, PaysFee, WeighData},
    traits::IsSubType,
    weights::Weight,
};
use frame_system::ensure_signed;
use log::info;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Bounded, DispatchInfoOf, SaturatedConversion, Saturating, SignedExtension},
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
};

pub use pallet::*;

pub mod weights;
pub use weights::*;

type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
const MILLICENTS: u32 = 1_000_000_000;

struct WeightForSetDummy<T: pallet_balances::Config>(BalanceOf<T>);

impl<T: pallet_balances::Config> WeighData<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
    fn weigh_data(&self, target: (&BalanceOf<T>,)) -> Weight {
        let multiplier = self.0;
        // *target.0 is the amount passed into the extrinsic
        let cents = *target.0 / <BalanceOf<T>>::from(MILLICENTS);
        Weight::from_parts((cents * multiplier).saturated_into::<u64>(), 0)
    }
}

impl<T: pallet_balances::Config> ClassifyDispatch<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
    fn classify_dispatch(&self, target: (&BalanceOf<T>,)) -> DispatchClass {
        if *target.0 > <BalanceOf<T>>::from(1000u32) {
            DispatchClass::Operational
        } else {
            DispatchClass::Normal
        }
    }
}

impl<T: pallet_balances::Config> PaysFee<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
    fn pays_fee(&self, _target: (&BalanceOf<T>,)) -> Pays {
        Pays::Yes
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        #[pallet::constant]
        type MagicNumber: Get<Self::Balance>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }

        fn on_finalize(_n: BlockNumberFor<T>) {}

        fn offchain_worker(_n: BlockNumberFor<T>) {}
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn cube_dummy(origin: OriginFor<T>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            <Dummy<T>>::mutate(|dummy| {
                *dummy = dummy.and_then(|d| Some(d.saturating_pow(3_usize)));
            });

            Self::deposit_event(Event::AccumulateDummy);

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(WeightForSetDummy::<T>(<BalanceOf<T>>::from(100u32)))]
        pub fn set_dummy(
            origin: OriginFor<T>,
            #[pallet::compact] new_value: T::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;

            info!("New value is now: {:?}", new_value);

            <Dummy<T>>::put(new_value);

            Self::deposit_event(Event::SetDummy { balance: new_value });

            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AccumulateDummy,
        SetDummy {
            balance: BalanceOf<T>,
        },
        SetBar {
            account: T::AccountId,
            balance: BalanceOf<T>,
        },
    }

    #[pallet::storage]
    pub(super) type Dummy<T: Config> = StorageValue<_, T::Balance>;

    #[pallet::storage]
    pub type CountedMap<T> = CountedStorageMap<_, Blake2_128Concat, u8, u16>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub dummy: T::Balance,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            <Dummy<T>>::put(self.dummy);
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct WatchDummy<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> core::fmt::Debug for WatchDummy<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "WatchDummy")
    }
}

impl<T: Config + Send + Sync> SignedExtension for WatchDummy<T>
where
    <T as frame_system::Config>::RuntimeCall: IsSubType<Call<T>>,
{
    const IDENTIFIER: &'static str = "WatchDummy";
    type AccountId = T::AccountId;
    type Call = <T as frame_system::Config>::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> core::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        self.validate(who, call, info, len).map(|_| ())
    }

    fn validate(
        &self,
        _who: &Self::AccountId,
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        if len > 200 {
            return InvalidTransaction::ExhaustsResources.into();
        }

        match call.is_sub_type() {
            Some(Call::set_dummy { .. }) => {
                sp_runtime::print("set_dummy was received.");

                let valid_tx = ValidTransaction {
                    priority: Bounded::max_value(),
                    ..Default::default()
                };
                Ok(valid_tx)
            }
            _ => Ok(Default::default()),
        }
    }
}
