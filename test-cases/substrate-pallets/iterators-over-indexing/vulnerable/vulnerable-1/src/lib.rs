#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]

extern crate alloc;

use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    dispatch::{ClassifyDispatch, DispatchClass, DispatchResult, Pays, PaysFee, WeighData},
    traits::IsSubType,
    weights::Weight,
};
use frame_system::ensure_signed;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Bounded, DispatchInfoOf, SaturatedConversion, SignedExtension},
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

        #[pallet::constant]
        type Count: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        VectorFull,
    }

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
        pub fn insert_dummy(origin: OriginFor<T>, value: u32) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            if let Some(v) = <Dummy<T>>::get() {
                if v.len() >= 128 {
                    Err(Error::<T>::VectorFull)?;
                }
            }

            <Dummy<T>>::mutate(|dummy| {
                if dummy.is_none() {
                    let mut temp = BoundedVec::<u32, T::Count>::new();
                    let _ = temp.try_push(value);
                    *dummy = Some(temp);
                } else {
                    let _ = dummy.as_mut().unwrap().try_push(value);
                }
            });

            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn set_sum(origin: OriginFor<T>) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            let mut new_sum = 0_u32;

            if let Some(v) = <Dummy<T>>::get() {
                for i in 0..128 {
                    new_sum += v[i];
                }
            }

            <Sum<T>>::mutate(|sum| {
                *sum = Some(new_sum);
            });

            Ok(())
        }
    }

    #[pallet::event]
    pub enum Event<T: Config> {}

    #[pallet::storage]
    pub(super) type Dummy<T: Config> = StorageValue<_, BoundedVec<u32, T::Count>>;

    #[pallet::storage]
    pub(super) type Sum<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    pub type CountedMap<T> = CountedStorageMap<_, Blake2_128Concat, u8, u16>;
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
            Some(Call::insert_dummy { .. }) => {
                sp_runtime::print("insert_dummy was received.");

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
