// lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod tests;

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
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn example_storage)]
    pub type ExampleStorage<T: Config> = StorageValue<_, u32>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // Add on_initialize if we need any per-block logic
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }
    }

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn unsafe_get_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let value = ExampleStorage::<T>::get().expect("Storage is not initialized");
            Self::deposit_event(Event::UnsafeGetStorage { who, value });
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn set_storage(origin: OriginFor<T>, new_value: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ExampleStorage::<T>::put(new_value);
            Self::deposit_event(Event::StorageSet {
                who,
                value: new_value,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        pub fn clear_storage(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ExampleStorage::<T>::kill();
            Self::deposit_event(Event::StorageCleared { who });
            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Storage was accessed through unsafe getter
        UnsafeGetStorage { who: T::AccountId, value: u32 },
        /// Storage value was set
        StorageSet { who: T::AccountId, value: u32 },
        /// Storage was cleared
        StorageCleared { who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Value is not initialized
        NotInitialized,
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
                ExampleStorage::<T>::put(value);
            }
        }
    }
}
