// lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn balance_storage)]
    pub type BalanceStorage<T: Config> = StorageValue<_, u32>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn unsafe_check_balance(origin: OriginFor<T>, amount: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if BalanceStorage::<T>::get().unwrap_or(0) < amount {
                return Err(Error::<T>::InvalidBalance.into());
            }

            Self::deposit_event(Event::BalanceChecked { who, amount });
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn set_balance(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            BalanceStorage::<T>::put(new_value);
            Self::deposit_event(Event::BalanceSet {
                who,
                value: new_value,
            });
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BalanceChecked { who: T::AccountId, amount: u32 },
        BalanceSet { who: T::AccountId, value: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidBalance,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub initial_balance: Option<u32>,
        #[serde(skip)]
        pub _phantom: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(value) = self.initial_balance {
                BalanceStorage::<T>::put(value);
            }
        }
    }
}
