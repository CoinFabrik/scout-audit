#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;

use frame_support::traits::Currency;
pub use pallet::*;
pub mod weights;
use sp_runtime::traits::Zero;
pub use weights::*;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type BalanceOf2<T> = <T as pallet_balances::Config>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{pallet_prelude::*, traits::BuildGenesisConfig};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_balances::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn value)]
    pub type Value<T: Config> = StorageValue<_, u32>;

    #[pallet::storage]
    #[pallet::getter(fn balance_of)]
    pub type Balance<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn set_balance(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let sender_balance = Self::balance_of(&who);
            let amount_u32: u32 = amount.try_into().unwrap_or(u32::MAX);
            ensure!(sender_balance >= amount_u32, "Insufficient balance");

            if amount == Zero::zero() {
                return Err(Error::<T>::ZeroBalance.into());
            }
            Self::deposit_event(Event::BalanceSet { who, value: amount });
            Ok(())
        }
        #[pallet::call_index(1)]
        pub fn unsafe_check_balance(origin: OriginFor<T>, amount: BalanceOf2<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let sender_balance = Self::balance_of(&who);
            let amount_u32: u32 = amount.try_into().unwrap_or(u32::MAX);

            if amount == Zero::zero() {
                return Err(Error::<T>::ZeroBalance.into());
            }

            ensure!(sender_balance >= amount_u32, "Insufficient balance");
            Ok(())
        }
    }
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BalanceChecked {
            who: T::AccountId,
            amount: u32,
        },
        BalanceSet {
            who: T::AccountId,
            value: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ZeroBalance,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub initial_value: Option<u32>,
        #[serde(skip)]
        pub _phantom: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(value) = self.initial_value {
                Value::<T>::put(value);
            }
        }
    }
}
