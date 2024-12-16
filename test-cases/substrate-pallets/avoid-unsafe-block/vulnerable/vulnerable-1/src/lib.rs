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
    #[pallet::getter(fn stored_value)]
    pub type DataStorage<T: Config> = StorageValue<_, u8, ValueQuery>;

    #[pallet::call(weight(<T as Config>::WeightInfo))]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn process_data(origin: OriginFor<T>, input: u8) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let result = unsafe {
                let ptr: *const u8 = &input;
                let value = *ptr;
                value.rotate_left(2).wrapping_add(1)
            };

            DataStorage::<T>::set(result);

            Self::deposit_event(Event::DataProcessed { who, value: result });

            Ok(())
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        DataProcessed { who: T::AccountId, value: u8 },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProcessingFailed,
    }

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        pub initial_value: Option<u8>,
        #[serde(skip)]
        pub _phantom: PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            if let Some(value) = self.initial_value {
                DataStorage::<T>::put(value);
            }
        }
    }
}
