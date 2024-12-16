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
    #[pallet::getter(fn value)]
    pub type Value<T: Config> = StorageValue<_, u32>;

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn unsafe_check_value(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let stored_value = Value::<T>::get().unwrap_or_default();
            if stored_value < threshold {
                panic!("Value is too low!");
            }

            Self::deposit_event(Event::ValueChecked {
                who,
                value: stored_value,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn set_value(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Value::<T>::put(new_value);
            Self::deposit_event(Event::ValueSet {
                who,
                value: new_value,
            });
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ValueChecked { who: T::AccountId, value: u32 },
        ValueSet { who: T::AccountId, value: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        ValueTooLow,
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
