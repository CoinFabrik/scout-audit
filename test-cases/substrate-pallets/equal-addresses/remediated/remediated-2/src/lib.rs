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
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn balance_of)]
    pub type Balance<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn check_balance(
            origin1: OriginFor<T>,
            from1: T::AccountId,
            amount: u32,
        ) -> DispatchResult {
            let origin1 = ensure_signed(origin1)?;

            let sender_balance = Self::balance_of(&from1);

            if from1 == origin1 {
                return Err(Error::<T>::SameAddresses.into());
            }

            ensure!(sender_balance >= amount, "Insufficient balance");

            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Transfer {
            from: T::AccountId,
            to: T::AccountId,
            amount: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        SameAddresses,
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
        fn build(&self) {}
    }
}
