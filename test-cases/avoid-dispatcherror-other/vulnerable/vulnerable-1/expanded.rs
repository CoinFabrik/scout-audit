#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
extern crate alloc;
use alloc::vec::Vec;
use codec::{Decode, Encode};
use core::marker::PhantomData;
use frame_support::{
    dispatch::{
        ClassifyDispatch, DispatchClass, DispatchResult, Pays, PaysFee, WeighData,
    },
    traits::IsSubType, weights::Weight,
};
use frame_system::ensure_signed;
use log::info;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{Bounded, DispatchInfoOf, SaturatedConversion, Saturating, SignedExtension},
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError,
        ValidTransaction,
    },
};
pub use pallet::*;
pub mod weights {
    #![allow(unused_parens)]
    #![allow(unused_imports)]
    use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
    use core::marker::PhantomData;
    pub trait WeightInfo {
        fn set_dummy_benchmark() -> Weight;
        fn accumulate_dummy() -> Weight;
        fn sort_vector(x: u32) -> Weight;
    }
    pub struct SubstrateWeight<T>(PhantomData<T>);
    impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
        fn set_dummy_benchmark() -> Weight {
            Weight::from_parts(19_000_000_u64, 0)
                .saturating_add(T::DbWeight::get().writes(1_u64))
        }
        fn accumulate_dummy() -> Weight {
            Weight::from_parts(18_000_000_u64, 0)
                .saturating_add(T::DbWeight::get().reads(1_u64))
                .saturating_add(T::DbWeight::get().writes(1_u64))
        }
        fn sort_vector(x: u32) -> Weight {
            Weight::from_parts(0_u64, 0)
                .saturating_add(Weight::from_parts(520_u64, 0).saturating_mul(x as u64))
        }
    }
    impl WeightInfo for () {
        fn set_dummy_benchmark() -> Weight {
            Weight::from_parts(19_000_000_u64, 0)
                .saturating_add(RocksDbWeight::get().writes(1_u64))
        }
        fn accumulate_dummy() -> Weight {
            Weight::from_parts(18_000_000_u64, 0)
                .saturating_add(RocksDbWeight::get().reads(1_u64))
                .saturating_add(RocksDbWeight::get().writes(1_u64))
        }
        fn sort_vector(x: u32) -> Weight {
            Weight::from_parts(0_u64, 0)
                .saturating_add(Weight::from_parts(520_u64, 0).saturating_mul(x as u64))
        }
    }
}
pub use weights::*;
type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
const MILLICENTS: u32 = 1_000_000_000;
struct WeightForSetDummy<T: pallet_balances::Config>(BalanceOf<T>);
impl<T: pallet_balances::Config> WeighData<(&BalanceOf<T>,)> for WeightForSetDummy<T> {
    fn weigh_data(&self, target: (&BalanceOf<T>,)) -> Weight {
        let multiplier = self.0;
        let cents = *target.0 / <BalanceOf<T>>::from(MILLICENTS);
        Weight::from_parts((cents * multiplier).saturated_into::<u64>(), 0)
    }
}
impl<T: pallet_balances::Config> ClassifyDispatch<(&BalanceOf<T>,)>
for WeightForSetDummy<T> {
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
/**The `pallet` module in each FRAME pallet hosts the most important items needed
to construct this pallet.

The main components of this pallet are:
- [`Pallet`], which implements all of the dispatchable extrinsics of the pallet, among
other public functions.
	- The subset of the functions that are dispatchable can be identified either in the
	[`dispatchables`] module or in the [`Call`] enum.
- [`storage_types`], which contains the list of all types that are representing a
storage item. Otherwise, all storage items are listed among [*Type Definitions*](#types).
- [`Config`], which contains the configuration trait of this pallet.
- [`Event`] and [`Error`], which are listed among the [*Enums*](#enums).
		*/
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    /**
Configuration trait of this pallet.

The main purpose of this trait is to act as an interface between this pallet and the runtime in
which it is embedded in. A type, function, or constant in this trait is essentially left to be
configured by the runtime that includes this pallet.

Consequently, a runtime that wants to include this pallet must implement this trait.*/
    pub trait Config: pallet_balances::Config + frame_system::Config {
        type MagicNumber: Get<Self::Balance>;
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }
    /**
				The `Pallet` struct, the main type that implements traits and standalone
				functions within the pallet.
			*/
    pub struct Pallet<T>(core::marker::PhantomData<(T)>);
    const _: () = {
        #[automatically_derived]
        impl<T> ::core::clone::Clone for Pallet<T> {
            fn clone(&self) -> Self {
                Self(::core::clone::Clone::clone(&self.0))
            }
        }
    };
    const _: () = {
        impl<T> ::core::cmp::Eq for Pallet<T> {}
    };
    const _: () = {
        #[automatically_derived]
        impl<T> ::core::cmp::PartialEq for Pallet<T> {
            fn eq(&self, other: &Self) -> bool {
                true && self.0 == other.0
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl<T> ::core::fmt::Debug for Pallet<T> {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                fmt.debug_tuple("Pallet").field(&self.0).finish()
            }
        }
    };
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }
        fn on_finalize(_n: BlockNumberFor<T>) {}
        fn offchain_worker(_n: BlockNumberFor<T>) {}
    }
    impl<T: Config> Pallet<T> {
        pub fn accumulate_dummy(
            origin: OriginFor<T>,
            increase_by: T::Balance,
        ) -> DispatchResult {
            frame_support::storage::with_storage_layer::<
                (),
                frame_support::pallet_prelude::DispatchError,
                _,
            >(|| {
                let _sender = ensure_signed(origin)?;
                <Dummy<
                    T,
                >>::mutate(|dummy| {
                    let new_dummy = dummy
                        .map_or(increase_by, |d| d.saturating_add(increase_by));
                    *dummy = Some(new_dummy);
                });
                Self::deposit_event(Event::AccumulateDummy {
                    balance: increase_by,
                });
                Ok(())
            })
        }
        pub fn set_dummy(origin: OriginFor<T>, new_value: T::Balance) -> DispatchResult {
            frame_support::storage::with_storage_layer::<
                (),
                frame_support::pallet_prelude::DispatchError,
                _,
            >(|| {
                ensure_root(origin)?;
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!("New value is now: {0:?}", new_value),
                            lvl,
                            &(
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                <Dummy<T>>::put(new_value);
                Self::deposit_event(Event::SetDummy {
                    balance: new_value,
                });
                Ok(())
            })
        }
    }
    ///The `Event` enum of this pallet
    #[scale_info(skip_type_params(T), capture_docs = "always")]
    pub enum Event<T: Config> {
        AccumulateDummy { balance: BalanceOf<T> },
        SetDummy { balance: BalanceOf<T> },
        SetBar { account: T::AccountId, balance: BalanceOf<T> },
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(::core::marker::PhantomData<(T)>, frame_support::Never),
    }
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::clone::Clone for Event<T> {
            fn clone(&self) -> Self {
                match self {
                    Self::AccumulateDummy { ref balance } => {
                        Self::AccumulateDummy {
                            balance: ::core::clone::Clone::clone(balance),
                        }
                    }
                    Self::SetDummy { ref balance } => {
                        Self::SetDummy {
                            balance: ::core::clone::Clone::clone(balance),
                        }
                    }
                    Self::SetBar { ref account, ref balance } => {
                        Self::SetBar {
                            account: ::core::clone::Clone::clone(account),
                            balance: ::core::clone::Clone::clone(balance),
                        }
                    }
                    Self::__Ignore(ref _0, ref _1) => {
                        Self::__Ignore(
                            ::core::clone::Clone::clone(_0),
                            ::core::clone::Clone::clone(_1),
                        )
                    }
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> ::core::cmp::Eq for Event<T> {}
    };
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::cmp::PartialEq for Event<T> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (
                        Self::AccumulateDummy { balance },
                        Self::AccumulateDummy { balance: _0 },
                    ) => true && balance == _0,
                    (Self::SetDummy { balance }, Self::SetDummy { balance: _0 }) => {
                        true && balance == _0
                    }
                    (
                        Self::SetBar { account, balance },
                        Self::SetBar { account: _0, balance: _1 },
                    ) => true && account == _0 && balance == _1,
                    (Self::__Ignore(_0, _1), Self::__Ignore(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (Self::AccumulateDummy { .. }, Self::SetDummy { .. }) => false,
                    (Self::AccumulateDummy { .. }, Self::SetBar { .. }) => false,
                    (Self::AccumulateDummy { .. }, Self::__Ignore { .. }) => false,
                    (Self::SetDummy { .. }, Self::AccumulateDummy { .. }) => false,
                    (Self::SetDummy { .. }, Self::SetBar { .. }) => false,
                    (Self::SetDummy { .. }, Self::__Ignore { .. }) => false,
                    (Self::SetBar { .. }, Self::AccumulateDummy { .. }) => false,
                    (Self::SetBar { .. }, Self::SetDummy { .. }) => false,
                    (Self::SetBar { .. }, Self::__Ignore { .. }) => false,
                    (Self::__Ignore { .. }, Self::AccumulateDummy { .. }) => false,
                    (Self::__Ignore { .. }, Self::SetDummy { .. }) => false,
                    (Self::__Ignore { .. }, Self::SetBar { .. }) => false,
                }
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::fmt::Debug for Event<T> {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Self::AccumulateDummy { ref balance } => {
                        fmt.debug_struct("Event::AccumulateDummy")
                            .field("balance", &balance)
                            .finish()
                    }
                    Self::SetDummy { ref balance } => {
                        fmt.debug_struct("Event::SetDummy")
                            .field("balance", &balance)
                            .finish()
                    }
                    Self::SetBar { ref account, ref balance } => {
                        fmt.debug_struct("Event::SetBar")
                            .field("account", &account)
                            .field("balance", &balance)
                            .finish()
                    }
                    Self::__Ignore(ref _0, ref _1) => {
                        fmt.debug_tuple("Event::__Ignore").field(&_0).field(&_1).finish()
                    }
                }
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Encode for Event<T>
        where
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            T::AccountId: ::codec::Encode,
            T::AccountId: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
        {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        Event::AccumulateDummy { ref balance } => {
                            0_usize.saturating_add(::codec::Encode::size_hint(balance))
                        }
                        Event::SetDummy { ref balance } => {
                            0_usize.saturating_add(::codec::Encode::size_hint(balance))
                        }
                        Event::SetBar { ref account, ref balance } => {
                            0_usize
                                .saturating_add(::codec::Encode::size_hint(account))
                                .saturating_add(::codec::Encode::size_hint(balance))
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    Event::AccumulateDummy { ref balance } => {
                        __codec_dest_edqy.push_byte(0usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(balance, __codec_dest_edqy);
                    }
                    Event::SetDummy { ref balance } => {
                        __codec_dest_edqy.push_byte(1usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(balance, __codec_dest_edqy);
                    }
                    Event::SetBar { ref account, ref balance } => {
                        __codec_dest_edqy.push_byte(2usize as ::core::primitive::u8);
                        ::codec::Encode::encode_to(account, __codec_dest_edqy);
                        ::codec::Encode::encode_to(balance, __codec_dest_edqy);
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl<T: Config> ::codec::EncodeLike for Event<T>
        where
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            T::AccountId: ::codec::Encode,
            T::AccountId: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
            BalanceOf<T>: ::codec::Encode,
        {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::codec::Decode for Event<T>
        where
            BalanceOf<T>: ::codec::Decode,
            BalanceOf<T>: ::codec::Decode,
            BalanceOf<T>: ::codec::Decode,
            BalanceOf<T>: ::codec::Decode,
            T::AccountId: ::codec::Decode,
            T::AccountId: ::codec::Decode,
            BalanceOf<T>: ::codec::Decode,
            BalanceOf<T>: ::codec::Decode,
        {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e.chain("Could not decode `Event`, failed to read variant byte")
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 0usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Event::<T>::AccumulateDummy {
                                balance: {
                                    let __codec_res_edqy = <BalanceOf<
                                        T,
                                    > as ::codec::Decode>::decode(__codec_input_edqy);
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e
                                                    .chain("Could not decode `Event::AccumulateDummy::balance`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                            })
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 1usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Event::<T>::SetDummy {
                                balance: {
                                    let __codec_res_edqy = <BalanceOf<
                                        T,
                                    > as ::codec::Decode>::decode(__codec_input_edqy);
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `Event::SetDummy::balance`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                            })
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy
                        == 2usize as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Event::<T>::SetBar {
                                account: {
                                    let __codec_res_edqy = <T::AccountId as ::codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `Event::SetBar::account`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                                balance: {
                                    let __codec_res_edqy = <BalanceOf<
                                        T,
                                    > as ::codec::Decode>::decode(__codec_input_edqy);
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `Event::SetBar::balance`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                            })
                        })();
                    }
                    _ => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Err(
                                <_ as ::core::convert::Into<
                                    _,
                                >>::into("Could not decode `Event`, variant doesn't exist"),
                            )
                        })();
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T: Config> ::scale_info::TypeInfo for Event<T>
        where
            BalanceOf<T>: ::scale_info::TypeInfo + 'static,
            BalanceOf<T>: ::scale_info::TypeInfo + 'static,
            T::AccountId: ::scale_info::TypeInfo + 'static,
            BalanceOf<T>: ::scale_info::TypeInfo + 'static,
            ::core::marker::PhantomData<(T)>: ::scale_info::TypeInfo + 'static,
            T: Config + 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new_with_replace(
                            "Event",
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            &[],
                        ),
                    )
                    .type_params(
                        <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([
                                ::scale_info::TypeParameter::new(
                                    "T",
                                    ::core::option::Option::None,
                                ),
                            ]),
                        ),
                    )
                    .docs_always(&["The `Event` enum of this pallet"])
                    .variant(
                        ::scale_info::build::Variants::new()
                            .variant(
                                "AccumulateDummy",
                                |v| {
                                    v
                                        .index(0usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::named()
                                                .field(|f| {
                                                    f
                                                        .ty::<BalanceOf<T>>()
                                                        .name("balance")
                                                        .type_name("BalanceOf<T>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "SetDummy",
                                |v| {
                                    v
                                        .index(1usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::named()
                                                .field(|f| {
                                                    f
                                                        .ty::<BalanceOf<T>>()
                                                        .name("balance")
                                                        .type_name("BalanceOf<T>")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "SetBar",
                                |v| {
                                    v
                                        .index(2usize as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::named()
                                                .field(|f| {
                                                    f
                                                        .ty::<T::AccountId>()
                                                        .name("account")
                                                        .type_name("T::AccountId")
                                                })
                                                .field(|f| {
                                                    f
                                                        .ty::<BalanceOf<T>>()
                                                        .name("balance")
                                                        .type_name("BalanceOf<T>")
                                                }),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    #[allow(type_alias_bounds)]
    ///
    ///Storage type is [`StorageValue`] with value type `T :: Balance`.
    pub(super) type Dummy<T: Config> = StorageValue<
        _GeneratedPrefixForStorageDummy<T>,
        T::Balance,
    >;
    #[allow(type_alias_bounds)]
    ///
    ///Storage type is [`StorageMap`] with key type `T :: AccountId` and value type `T :: Balance`.
    pub(super) type Bar<T: Config> = StorageMap<
        _GeneratedPrefixForStorageBar<T>,
        Blake2_128Concat,
        T::AccountId,
        T::Balance,
    >;
    #[allow(type_alias_bounds)]
    ///
    ///Storage type is [`StorageValue`] with value type `T :: Balance`.
    pub(super) type Foo<T: Config> = StorageValue<
        _GeneratedPrefixForStorageFoo<T>,
        T::Balance,
        ValueQuery,
    >;
    #[allow(type_alias_bounds)]
    ///
    ///Storage type is [`CountedStorageMap`] with key type u8 and value type u16.
    pub type CountedMap<T> = CountedStorageMap<
        _GeneratedPrefixForStorageCountedMap<T>,
        Blake2_128Concat,
        u8,
        u16,
    >;
    /**
					Can be used to configure the
					[genesis state](https://docs.substrate.io/build/genesis-configuration/)
					of this pallet.
					*/
    #[serde(rename_all = "camelCase")]
    #[serde(deny_unknown_fields)]
    #[serde(bound(serialize = ""))]
    #[serde(bound(deserialize = ""))]
    #[serde(crate = "frame_support::__private::serde")]
    pub struct GenesisConfig<T: Config> {
        pub dummy: T::Balance,
        pub bar: Vec<(T::AccountId, T::Balance)>,
        pub foo: T::Balance,
    }
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        use frame_support::__private::serde as _serde;
        #[automatically_derived]
        impl<T: Config> frame_support::__private::serde::Serialize for GenesisConfig<T> {
            fn serialize<__S>(
                &self,
                __serializer: __S,
            ) -> frame_support::__private::serde::__private::Result<__S::Ok, __S::Error>
            where
                __S: frame_support::__private::serde::Serializer,
            {
                let mut __serde_state = _serde::Serializer::serialize_struct(
                    __serializer,
                    "GenesisConfig",
                    false as usize + 1 + 1 + 1,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "dummy",
                    &self.dummy,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "bar",
                    &self.bar,
                )?;
                _serde::ser::SerializeStruct::serialize_field(
                    &mut __serde_state,
                    "foo",
                    &self.foo,
                )?;
                _serde::ser::SerializeStruct::end(__serde_state)
            }
        }
    };
    #[doc(hidden)]
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        use frame_support::__private::serde as _serde;
        #[automatically_derived]
        impl<'de, T: Config> frame_support::__private::serde::Deserialize<'de>
        for GenesisConfig<T> {
            fn deserialize<__D>(
                __deserializer: __D,
            ) -> frame_support::__private::serde::__private::Result<Self, __D::Error>
            where
                __D: frame_support::__private::serde::Deserializer<'de>,
            {
                #[allow(non_camel_case_types)]
                #[doc(hidden)]
                enum __Field {
                    __field0,
                    __field1,
                    __field2,
                }
                #[doc(hidden)]
                struct __FieldVisitor;
                impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                    type Value = __Field;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "field identifier",
                        )
                    }
                    fn visit_u64<__E>(
                        self,
                        __value: u64,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            0u64 => _serde::__private::Ok(__Field::__field0),
                            1u64 => _serde::__private::Ok(__Field::__field1),
                            2u64 => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::invalid_value(
                                        _serde::de::Unexpected::Unsigned(__value),
                                        &"field index 0 <= i < 3",
                                    ),
                                )
                            }
                        }
                    }
                    fn visit_str<__E>(
                        self,
                        __value: &str,
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            "dummy" => _serde::__private::Ok(__Field::__field0),
                            "bar" => _serde::__private::Ok(__Field::__field1),
                            "foo" => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_field(__value, FIELDS),
                                )
                            }
                        }
                    }
                    fn visit_bytes<__E>(
                        self,
                        __value: &[u8],
                    ) -> _serde::__private::Result<Self::Value, __E>
                    where
                        __E: _serde::de::Error,
                    {
                        match __value {
                            b"dummy" => _serde::__private::Ok(__Field::__field0),
                            b"bar" => _serde::__private::Ok(__Field::__field1),
                            b"foo" => _serde::__private::Ok(__Field::__field2),
                            _ => {
                                let __value = &_serde::__private::from_utf8_lossy(__value);
                                _serde::__private::Err(
                                    _serde::de::Error::unknown_field(__value, FIELDS),
                                )
                            }
                        }
                    }
                }
                impl<'de> _serde::Deserialize<'de> for __Field {
                    #[inline]
                    fn deserialize<__D>(
                        __deserializer: __D,
                    ) -> _serde::__private::Result<Self, __D::Error>
                    where
                        __D: _serde::Deserializer<'de>,
                    {
                        _serde::Deserializer::deserialize_identifier(
                            __deserializer,
                            __FieldVisitor,
                        )
                    }
                }
                #[doc(hidden)]
                struct __Visitor<'de, T: Config> {
                    marker: _serde::__private::PhantomData<GenesisConfig<T>>,
                    lifetime: _serde::__private::PhantomData<&'de ()>,
                }
                impl<'de, T: Config> _serde::de::Visitor<'de> for __Visitor<'de, T> {
                    type Value = GenesisConfig<T>;
                    fn expecting(
                        &self,
                        __formatter: &mut _serde::__private::Formatter,
                    ) -> _serde::__private::fmt::Result {
                        _serde::__private::Formatter::write_str(
                            __formatter,
                            "struct GenesisConfig",
                        )
                    }
                    #[inline]
                    fn visit_seq<__A>(
                        self,
                        mut __seq: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::SeqAccess<'de>,
                    {
                        let __field0 = match _serde::de::SeqAccess::next_element::<
                            T::Balance,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        0usize,
                                        &"struct GenesisConfig with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field1 = match _serde::de::SeqAccess::next_element::<
                            Vec<(T::AccountId, T::Balance)>,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        1usize,
                                        &"struct GenesisConfig with 3 elements",
                                    ),
                                );
                            }
                        };
                        let __field2 = match _serde::de::SeqAccess::next_element::<
                            T::Balance,
                        >(&mut __seq)? {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(
                                    _serde::de::Error::invalid_length(
                                        2usize,
                                        &"struct GenesisConfig with 3 elements",
                                    ),
                                );
                            }
                        };
                        _serde::__private::Ok(GenesisConfig {
                            dummy: __field0,
                            bar: __field1,
                            foo: __field2,
                        })
                    }
                    #[inline]
                    fn visit_map<__A>(
                        self,
                        mut __map: __A,
                    ) -> _serde::__private::Result<Self::Value, __A::Error>
                    where
                        __A: _serde::de::MapAccess<'de>,
                    {
                        let mut __field0: _serde::__private::Option<T::Balance> = _serde::__private::None;
                        let mut __field1: _serde::__private::Option<
                            Vec<(T::AccountId, T::Balance)>,
                        > = _serde::__private::None;
                        let mut __field2: _serde::__private::Option<T::Balance> = _serde::__private::None;
                        while let _serde::__private::Some(__key) = _serde::de::MapAccess::next_key::<
                            __Field,
                        >(&mut __map)? {
                            match __key {
                                __Field::__field0 => {
                                    if _serde::__private::Option::is_some(&__field0) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("dummy"),
                                        );
                                    }
                                    __field0 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<T::Balance>(&mut __map)?,
                                    );
                                }
                                __Field::__field1 => {
                                    if _serde::__private::Option::is_some(&__field1) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("bar"),
                                        );
                                    }
                                    __field1 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<
                                            Vec<(T::AccountId, T::Balance)>,
                                        >(&mut __map)?,
                                    );
                                }
                                __Field::__field2 => {
                                    if _serde::__private::Option::is_some(&__field2) {
                                        return _serde::__private::Err(
                                            <__A::Error as _serde::de::Error>::duplicate_field("foo"),
                                        );
                                    }
                                    __field2 = _serde::__private::Some(
                                        _serde::de::MapAccess::next_value::<T::Balance>(&mut __map)?,
                                    );
                                }
                            }
                        }
                        let __field0 = match __field0 {
                            _serde::__private::Some(__field0) => __field0,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("dummy")?
                            }
                        };
                        let __field1 = match __field1 {
                            _serde::__private::Some(__field1) => __field1,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("bar")?
                            }
                        };
                        let __field2 = match __field2 {
                            _serde::__private::Some(__field2) => __field2,
                            _serde::__private::None => {
                                _serde::__private::de::missing_field("foo")?
                            }
                        };
                        _serde::__private::Ok(GenesisConfig {
                            dummy: __field0,
                            bar: __field1,
                            foo: __field2,
                        })
                    }
                }
                #[doc(hidden)]
                const FIELDS: &'static [&'static str] = &["dummy", "bar", "foo"];
                _serde::Deserializer::deserialize_struct(
                    __deserializer,
                    "GenesisConfig",
                    FIELDS,
                    __Visitor {
                        marker: _serde::__private::PhantomData::<GenesisConfig<T>>,
                        lifetime: _serde::__private::PhantomData,
                    },
                )
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::default::Default for GenesisConfig<T> {
            fn default() -> Self {
                Self {
                    dummy: ::core::default::Default::default(),
                    bar: ::core::default::Default::default(),
                    foo: ::core::default::Default::default(),
                }
            }
        }
    };
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            <Dummy<T>>::put(self.dummy);
            for (a, b) in &self.bar {
                <Bar<T>>::insert(a, b);
            }
            <Foo<T>>::put(self.foo);
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn pallet_documentation_metadata() -> frame_support::__private::Vec<
            &'static str,
        > {
            ::alloc::vec::Vec::new()
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn pallet_constants_metadata() -> frame_support::__private::Vec<
            frame_support::__private::metadata_ir::PalletConstantMetadataIR,
        > {
            <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    {
                        frame_support::__private::metadata_ir::PalletConstantMetadataIR {
                            name: "MagicNumber",
                            ty: frame_support::__private::scale_info::meta_type::<
                                T::Balance,
                            >(),
                            value: {
                                let value = <<T as Config>::MagicNumber as frame_support::traits::Get<
                                    T::Balance,
                                >>::get();
                                frame_support::__private::codec::Encode::encode(&value)
                            },
                            docs: ::alloc::vec::Vec::new(),
                        }
                    },
                ]),
            )
        }
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn error_metadata() -> Option<
            frame_support::__private::metadata_ir::PalletErrorMetadataIR,
        > {
            None
        }
    }
    /// Type alias to `Pallet`, to be used by `construct_runtime`.
    ///
    /// Generated by `pallet` attribute macro.
    #[deprecated(note = "use `Pallet` instead")]
    #[allow(dead_code)]
    pub type Module<T> = Pallet<T>;
    impl<T: Config> frame_support::traits::GetStorageVersion for Pallet<T> {
        type InCodeStorageVersion = frame_support::traits::NoStorageVersionSet;
        fn in_code_storage_version() -> Self::InCodeStorageVersion {
            core::default::Default::default()
        }
        fn on_chain_storage_version() -> frame_support::traits::StorageVersion {
            frame_support::traits::StorageVersion::get::<Self>()
        }
    }
    impl<T: Config> frame_support::traits::OnGenesis for Pallet<T> {
        fn on_genesis() {
            let storage_version: frame_support::traits::StorageVersion = core::default::Default::default();
            storage_version.put::<Self>();
        }
    }
    impl<T: Config> frame_support::traits::PalletInfoAccess for Pallet<T> {
        fn index() -> usize {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<
                Self,
            >()
                .expect(
                    "Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime",
                )
        }
        fn name() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Self,
            >()
                .expect(
                    "Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime",
                )
        }
        fn name_hash() -> [u8; 16] {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name_hash::<
                Self,
            >()
                .expect(
                    "Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime",
                )
        }
        fn module_name() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::module_name::<
                Self,
            >()
                .expect(
                    "Pallet is part of the runtime because pallet `Config` trait is \
						implemented by the runtime",
                )
        }
        fn crate_version() -> frame_support::traits::CrateVersion {
            frame_support::traits::CrateVersion {
                major: 27u16,
                minor: 0u8,
                patch: 0u8,
            }
        }
    }
    impl<T: Config> frame_support::traits::PalletsInfoAccess for Pallet<T> {
        fn count() -> usize {
            1
        }
        fn infos() -> frame_support::__private::Vec<
            frame_support::traits::PalletInfoData,
        > {
            use frame_support::traits::PalletInfoAccess;
            let item = frame_support::traits::PalletInfoData {
                index: Self::index(),
                name: Self::name(),
                module_name: Self::module_name(),
                crate_version: Self::crate_version(),
            };
            <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([item]))
        }
    }
    impl<T: Config> frame_support::traits::StorageInfoTrait for Pallet<T> {
        fn storage_info() -> frame_support::__private::Vec<
            frame_support::traits::StorageInfo,
        > {
            #[allow(unused_mut)]
            let mut res = ::alloc::vec::Vec::new();
            {
                let mut storage_info = <Dummy<
                    T,
                > as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info = <Bar<
                    T,
                > as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info = <Foo<
                    T,
                > as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            {
                let mut storage_info = <CountedMap<
                    T,
                > as frame_support::traits::StorageInfoTrait>::storage_info();
                res.append(&mut storage_info);
            }
            res
        }
    }
    use frame_support::traits::{
        StorageInfoTrait, TrackedStorageKey, WhitelistedStorageKeys,
    };
    impl<T: Config> WhitelistedStorageKeys for Pallet<T> {
        fn whitelisted_storage_keys() -> frame_support::__private::Vec<
            TrackedStorageKey,
        > {
            use frame_support::__private::vec;
            ::alloc::vec::Vec::new()
        }
    }
    #[doc(hidden)]
    mod warnings {}
    #[allow(unused_imports)]
    #[doc(hidden)]
    pub mod __substrate_call_check {
        #[doc(hidden)]
        pub use __is_call_part_defined_0 as is_call_part_defined;
    }
    ///Contains a variant per dispatchable extrinsic that this pallet has.
    #[codec(encode_bound())]
    #[codec(decode_bound())]
    #[scale_info(skip_type_params(T), capture_docs = "always")]
    #[allow(non_camel_case_types)]
    pub enum Call<T: Config> {
        #[doc(hidden)]
        #[codec(skip)]
        __Ignore(::core::marker::PhantomData<(T,)>, frame_support::Never),
        #[codec(index = 0u8)]
        accumulate_dummy { #[allow(missing_docs)] increase_by: T::Balance },
        #[codec(index = 1u8)]
        set_dummy { #[allow(missing_docs)] #[codec(compact)] new_value: T::Balance },
    }
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::fmt::Debug for Call<T> {
            fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match *self {
                    Self::__Ignore(ref _0, ref _1) => {
                        fmt.debug_tuple("Call::__Ignore").field(&_0).field(&_1).finish()
                    }
                    Self::accumulate_dummy { ref increase_by } => {
                        fmt.debug_struct("Call::accumulate_dummy")
                            .field("increase_by", &increase_by)
                            .finish()
                    }
                    Self::set_dummy { ref new_value } => {
                        fmt.debug_struct("Call::set_dummy")
                            .field("new_value", &new_value)
                            .finish()
                    }
                }
            }
        }
    };
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::clone::Clone for Call<T> {
            fn clone(&self) -> Self {
                match self {
                    Self::__Ignore(ref _0, ref _1) => {
                        Self::__Ignore(
                            ::core::clone::Clone::clone(_0),
                            ::core::clone::Clone::clone(_1),
                        )
                    }
                    Self::accumulate_dummy { ref increase_by } => {
                        Self::accumulate_dummy {
                            increase_by: ::core::clone::Clone::clone(increase_by),
                        }
                    }
                    Self::set_dummy { ref new_value } => {
                        Self::set_dummy {
                            new_value: ::core::clone::Clone::clone(new_value),
                        }
                    }
                }
            }
        }
    };
    const _: () = {
        impl<T: Config> ::core::cmp::Eq for Call<T> {}
    };
    const _: () = {
        #[automatically_derived]
        impl<T: Config> ::core::cmp::PartialEq for Call<T> {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::__Ignore(_0, _1), Self::__Ignore(_0_other, _1_other)) => {
                        true && _0 == _0_other && _1 == _1_other
                    }
                    (
                        Self::accumulate_dummy { increase_by },
                        Self::accumulate_dummy { increase_by: _0 },
                    ) => true && increase_by == _0,
                    (
                        Self::set_dummy { new_value },
                        Self::set_dummy { new_value: _0 },
                    ) => true && new_value == _0,
                    (Self::__Ignore { .. }, Self::accumulate_dummy { .. }) => false,
                    (Self::__Ignore { .. }, Self::set_dummy { .. }) => false,
                    (Self::accumulate_dummy { .. }, Self::__Ignore { .. }) => false,
                    (Self::accumulate_dummy { .. }, Self::set_dummy { .. }) => false,
                    (Self::set_dummy { .. }, Self::__Ignore { .. }) => false,
                    (Self::set_dummy { .. }, Self::accumulate_dummy { .. }) => false,
                }
            }
        }
    };
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl<T: Config> ::codec::Encode for Call<T> {
            fn size_hint(&self) -> usize {
                1_usize
                    + match *self {
                        Call::accumulate_dummy { ref increase_by } => {
                            0_usize
                                .saturating_add(::codec::Encode::size_hint(increase_by))
                        }
                        Call::set_dummy { ref new_value } => {
                            0_usize
                                .saturating_add(
                                    ::codec::Encode::size_hint(
                                        &<<T::Balance as ::codec::HasCompact>::Type as ::codec::EncodeAsRef<
                                            '_,
                                            T::Balance,
                                        >>::RefType::from(new_value),
                                    ),
                                )
                        }
                        _ => 0_usize,
                    }
            }
            fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
                &self,
                __codec_dest_edqy: &mut __CodecOutputEdqy,
            ) {
                match *self {
                    Call::accumulate_dummy { ref increase_by } => {
                        __codec_dest_edqy.push_byte(0u8 as ::core::primitive::u8);
                        ::codec::Encode::encode_to(increase_by, __codec_dest_edqy);
                    }
                    Call::set_dummy { ref new_value } => {
                        __codec_dest_edqy.push_byte(1u8 as ::core::primitive::u8);
                        {
                            ::codec::Encode::encode_to(
                                &<<T::Balance as ::codec::HasCompact>::Type as ::codec::EncodeAsRef<
                                    '_,
                                    T::Balance,
                                >>::RefType::from(new_value),
                                __codec_dest_edqy,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        #[automatically_derived]
        impl<T: Config> ::codec::EncodeLike for Call<T> {}
    };
    #[allow(deprecated)]
    const _: () = {
        #[allow(non_camel_case_types)]
        #[automatically_derived]
        impl<T: Config> ::codec::Decode for Call<T> {
            fn decode<__CodecInputEdqy: ::codec::Input>(
                __codec_input_edqy: &mut __CodecInputEdqy,
            ) -> ::core::result::Result<Self, ::codec::Error> {
                match __codec_input_edqy
                    .read_byte()
                    .map_err(|e| {
                        e.chain("Could not decode `Call`, failed to read variant byte")
                    })?
                {
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy == 0u8 as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Call::<T>::accumulate_dummy {
                                increase_by: {
                                    let __codec_res_edqy = <T::Balance as ::codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e
                                                    .chain(
                                                        "Could not decode `Call::accumulate_dummy::increase_by`",
                                                    ),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy
                                        }
                                    }
                                },
                            })
                        })();
                    }
                    #[allow(clippy::unnecessary_cast)]
                    __codec_x_edqy if __codec_x_edqy == 1u8 as ::core::primitive::u8 => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Ok(Call::<T>::set_dummy {
                                new_value: {
                                    let __codec_res_edqy = <<T::Balance as ::codec::HasCompact>::Type as ::codec::Decode>::decode(
                                        __codec_input_edqy,
                                    );
                                    match __codec_res_edqy {
                                        ::core::result::Result::Err(e) => {
                                            return ::core::result::Result::Err(
                                                e.chain("Could not decode `Call::set_dummy::new_value`"),
                                            );
                                        }
                                        ::core::result::Result::Ok(__codec_res_edqy) => {
                                            __codec_res_edqy.into()
                                        }
                                    }
                                },
                            })
                        })();
                    }
                    _ => {
                        #[allow(clippy::redundant_closure_call)]
                        return (move || {
                            ::core::result::Result::Err(
                                <_ as ::core::convert::Into<
                                    _,
                                >>::into("Could not decode `Call`, variant doesn't exist"),
                            )
                        })();
                    }
                }
            }
        }
    };
    #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
    const _: () = {
        impl<T: Config> ::scale_info::TypeInfo for Call<T>
        where
            ::core::marker::PhantomData<(T,)>: ::scale_info::TypeInfo + 'static,
            T::Balance: ::scale_info::TypeInfo + 'static,
            T::Balance: ::scale_info::scale::HasCompact,
            T: Config + 'static,
        {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(
                        ::scale_info::Path::new_with_replace(
                            "Call",
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            &[],
                        ),
                    )
                    .type_params(
                        <[_]>::into_vec(
                            #[rustc_box]
                            ::alloc::boxed::Box::new([
                                ::scale_info::TypeParameter::new(
                                    "T",
                                    ::core::option::Option::None,
                                ),
                            ]),
                        ),
                    )
                    .docs_always(
                        &[
                            "Contains a variant per dispatchable extrinsic that this pallet has.",
                        ],
                    )
                    .variant(
                        ::scale_info::build::Variants::new()
                            .variant(
                                "accumulate_dummy",
                                |v| {
                                    v
                                        .index(0u8 as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::named()
                                                .field(|f| {
                                                    f
                                                        .ty::<T::Balance>()
                                                        .name("increase_by")
                                                        .type_name("T::Balance")
                                                }),
                                        )
                                },
                            )
                            .variant(
                                "set_dummy",
                                |v| {
                                    v
                                        .index(1u8 as ::core::primitive::u8)
                                        .fields(
                                            ::scale_info::build::Fields::named()
                                                .field(|f| {
                                                    f
                                                        .compact::<T::Balance>()
                                                        .name("new_value")
                                                        .type_name("T::Balance")
                                                }),
                                        )
                                },
                            ),
                    )
            }
        }
    };
    impl<T: Config> Call<T> {
        ///Create a call with the variant `accumulate_dummy`.
        pub fn new_call_variant_accumulate_dummy(increase_by: T::Balance) -> Self {
            Self::accumulate_dummy {
                increase_by,
            }
        }
        ///Create a call with the variant `set_dummy`.
        pub fn new_call_variant_set_dummy(new_value: T::Balance) -> Self {
            Self::set_dummy { new_value }
        }
    }
    impl<T: Config> frame_support::dispatch::GetDispatchInfo for Call<T> {
        fn get_dispatch_info(&self) -> frame_support::dispatch::DispatchInfo {
            match *self {
                Self::accumulate_dummy { ref increase_by } => {
                    let __pallet_base_weight = <T as Config>::WeightInfo::accumulate_dummy();
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<
                        (&T::Balance,),
                    >>::weigh_data(&__pallet_base_weight, (increase_by,));
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<
                        (&T::Balance,),
                    >>::classify_dispatch(&__pallet_base_weight, (increase_by,));
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<
                        (&T::Balance,),
                    >>::pays_fee(&__pallet_base_weight, (increase_by,));
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::set_dummy { ref new_value } => {
                    let __pallet_base_weight = WeightForSetDummy::<
                        T,
                    >(<BalanceOf<T>>::from(100u32));
                    let __pallet_weight = <dyn frame_support::dispatch::WeighData<
                        (&T::Balance,),
                    >>::weigh_data(&__pallet_base_weight, (new_value,));
                    let __pallet_class = <dyn frame_support::dispatch::ClassifyDispatch<
                        (&T::Balance,),
                    >>::classify_dispatch(&__pallet_base_weight, (new_value,));
                    let __pallet_pays_fee = <dyn frame_support::dispatch::PaysFee<
                        (&T::Balance,),
                    >>::pays_fee(&__pallet_base_weight, (new_value,));
                    frame_support::dispatch::DispatchInfo {
                        weight: __pallet_weight,
                        class: __pallet_class,
                        pays_fee: __pallet_pays_fee,
                    }
                }
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "internal error: entered unreachable code: {0}",
                            format_args!("__Ignore cannot be used"),
                        ),
                    );
                }
            }
        }
    }
    impl<T: Config> frame_support::dispatch::CheckIfFeeless for Call<T> {
        type Origin = frame_system::pallet_prelude::OriginFor<T>;
        #[allow(unused_variables)]
        fn is_feeless(&self, origin: &Self::Origin) -> bool {
            match *self {
                Self::accumulate_dummy { ref increase_by } => false,
                Self::set_dummy { ref new_value } => false,
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "internal error: entered unreachable code: {0}",
                            format_args!("__Ignore cannot be used"),
                        ),
                    );
                }
            }
        }
    }
    impl<T: Config> frame_support::traits::GetCallName for Call<T> {
        fn get_call_name(&self) -> &'static str {
            match *self {
                Self::accumulate_dummy { .. } => "accumulate_dummy",
                Self::set_dummy { .. } => "set_dummy",
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "internal error: entered unreachable code: {0}",
                            format_args!("__PhantomItem cannot be used."),
                        ),
                    );
                }
            }
        }
        fn get_call_names() -> &'static [&'static str] {
            &["accumulate_dummy", "set_dummy"]
        }
    }
    impl<T: Config> frame_support::traits::GetCallIndex for Call<T> {
        fn get_call_index(&self) -> u8 {
            match *self {
                Self::accumulate_dummy { .. } => 0u8,
                Self::set_dummy { .. } => 1u8,
                Self::__Ignore(_, _) => {
                    ::core::panicking::panic_fmt(
                        format_args!(
                            "internal error: entered unreachable code: {0}",
                            format_args!("__PhantomItem cannot be used."),
                        ),
                    );
                }
            }
        }
        fn get_call_indices() -> &'static [u8] {
            &[0u8, 1u8]
        }
    }
    impl<T: Config> frame_support::traits::UnfilteredDispatchable for Call<T> {
        type RuntimeOrigin = frame_system::pallet_prelude::OriginFor<T>;
        fn dispatch_bypass_filter(
            self,
            origin: Self::RuntimeOrigin,
        ) -> frame_support::dispatch::DispatchResultWithPostInfo {
            frame_support::dispatch_context::run_in_context(|| {
                match self {
                    Self::accumulate_dummy { increase_by } => {
                        let __within_span__ = {
                            use ::tracing::__macro_support::Callsite as _;
                            static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                static META: ::tracing::Metadata<'static> = {
                                    ::tracing_core::metadata::Metadata::new(
                                        "accumulate_dummy",
                                        "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                        ::tracing::Level::TRACE,
                                        ::core::option::Option::Some(
                                            "inconsistent-weight-name/vulnerable/vulnerable-1/src/lib.rs",
                                        ),
                                        ::core::option::Option::Some(61u32),
                                        ::core::option::Option::Some(
                                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                        ),
                                        ::tracing_core::field::FieldSet::new(
                                            &[],
                                            ::tracing_core::callsite::Identifier(&__CALLSITE),
                                        ),
                                        ::tracing::metadata::Kind::SPAN,
                                    )
                                };
                                ::tracing::callsite::DefaultCallsite::new(&META)
                            };
                            let mut interest = ::tracing::subscriber::Interest::never();
                            if ::tracing::Level::TRACE
                                <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                && ::tracing::Level::TRACE
                                    <= ::tracing::level_filters::LevelFilter::current()
                                && {
                                    interest = __CALLSITE.interest();
                                    !interest.is_never()
                                }
                                && ::tracing::__macro_support::__is_enabled(
                                    __CALLSITE.metadata(),
                                    interest,
                                )
                            {
                                let meta = __CALLSITE.metadata();
                                ::tracing::Span::new(
                                    meta,
                                    &{ meta.fields().value_set(&[]) },
                                )
                            } else {
                                let span = ::tracing::__macro_support::__disabled_span(
                                    __CALLSITE.metadata(),
                                );
                                if match ::tracing::Level::TRACE {
                                    ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                    ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                    ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                    ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                    _ => ::tracing::log::Level::Trace,
                                } <= ::tracing::log::STATIC_MAX_LEVEL
                                {
                                    if !::tracing::dispatcher::has_been_set() {
                                        {
                                            span.record_all(
                                                &{ __CALLSITE.metadata().fields().value_set(&[]) },
                                            );
                                        }
                                    } else {
                                        {}
                                    }
                                } else {
                                    {}
                                };
                                span
                            }
                        };
                        let __tracing_guard__ = __within_span__.enter();
                        <Pallet<T>>::accumulate_dummy(origin, increase_by)
                            .map(Into::into)
                            .map_err(Into::into)
                    }
                    Self::set_dummy { new_value } => {
                        let __within_span__ = {
                            use ::tracing::__macro_support::Callsite as _;
                            static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                                static META: ::tracing::Metadata<'static> = {
                                    ::tracing_core::metadata::Metadata::new(
                                        "set_dummy",
                                        "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                        ::tracing::Level::TRACE,
                                        ::core::option::Option::Some(
                                            "inconsistent-weight-name/vulnerable/vulnerable-1/src/lib.rs",
                                        ),
                                        ::core::option::Option::Some(61u32),
                                        ::core::option::Option::Some(
                                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                        ),
                                        ::tracing_core::field::FieldSet::new(
                                            &[],
                                            ::tracing_core::callsite::Identifier(&__CALLSITE),
                                        ),
                                        ::tracing::metadata::Kind::SPAN,
                                    )
                                };
                                ::tracing::callsite::DefaultCallsite::new(&META)
                            };
                            let mut interest = ::tracing::subscriber::Interest::never();
                            if ::tracing::Level::TRACE
                                <= ::tracing::level_filters::STATIC_MAX_LEVEL
                                && ::tracing::Level::TRACE
                                    <= ::tracing::level_filters::LevelFilter::current()
                                && {
                                    interest = __CALLSITE.interest();
                                    !interest.is_never()
                                }
                                && ::tracing::__macro_support::__is_enabled(
                                    __CALLSITE.metadata(),
                                    interest,
                                )
                            {
                                let meta = __CALLSITE.metadata();
                                ::tracing::Span::new(
                                    meta,
                                    &{ meta.fields().value_set(&[]) },
                                )
                            } else {
                                let span = ::tracing::__macro_support::__disabled_span(
                                    __CALLSITE.metadata(),
                                );
                                if match ::tracing::Level::TRACE {
                                    ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                                    ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                                    ::tracing::Level::INFO => ::tracing::log::Level::Info,
                                    ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                                    _ => ::tracing::log::Level::Trace,
                                } <= ::tracing::log::STATIC_MAX_LEVEL
                                {
                                    if !::tracing::dispatcher::has_been_set() {
                                        {
                                            span.record_all(
                                                &{ __CALLSITE.metadata().fields().value_set(&[]) },
                                            );
                                        }
                                    } else {
                                        {}
                                    }
                                } else {
                                    {}
                                };
                                span
                            }
                        };
                        let __tracing_guard__ = __within_span__.enter();
                        <Pallet<T>>::set_dummy(origin, new_value)
                            .map(Into::into)
                            .map_err(Into::into)
                    }
                    Self::__Ignore(_, _) => {
                        let _ = origin;
                        {
                            ::core::panicking::panic_fmt(
                                format_args!(
                                    "internal error: entered unreachable code: {0}",
                                    format_args!("__PhantomItem cannot be used."),
                                ),
                            );
                        };
                    }
                }
            })
        }
    }
    impl<T: Config> frame_support::dispatch::Callable<T> for Pallet<T> {
        type RuntimeCall = Call<T>;
    }
    impl<T: Config> Pallet<T> {
        #[allow(dead_code)]
        #[doc(hidden)]
        pub fn call_functions() -> frame_support::__private::metadata_ir::PalletCallMetadataIR {
            frame_support::__private::scale_info::meta_type::<Call<T>>().into()
        }
    }
    pub use __tt_error_token_1 as tt_error_token;
    #[doc(hidden)]
    pub mod __substrate_event_check {
        #[doc(hidden)]
        pub use __is_event_part_defined_2 as is_event_part_defined;
    }
    impl<T: Config> Pallet<T> {
        pub(super) fn deposit_event(event: Event<T>) {
            let event = <<T as Config>::RuntimeEvent as From<Event<T>>>::from(event);
            let event = <<T as Config>::RuntimeEvent as Into<
                <T as frame_system::Config>::RuntimeEvent,
            >>::into(event);
            <frame_system::Pallet<T>>::deposit_event(event)
        }
    }
    impl<T: Config> From<Event<T>> for () {
        fn from(_: Event<T>) {}
    }
    impl<T: Config> Pallet<T> {
        #[doc(hidden)]
        pub fn storage_metadata() -> frame_support::__private::metadata_ir::PalletStorageMetadataIR {
            frame_support::__private::metadata_ir::PalletStorageMetadataIR {
                prefix: <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                    Pallet<T>,
                >()
                    .expect(
                        "No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                    ),
                entries: {
                    #[allow(unused_mut)]
                    let mut entries = ::alloc::vec::Vec::new();
                    {
                        <Dummy<
                            T,
                        > as frame_support::storage::StorageEntryMetadataBuilder>::build_metadata(
                            ::alloc::vec::Vec::new(),
                            &mut entries,
                        );
                    }
                    {
                        <Bar<
                            T,
                        > as frame_support::storage::StorageEntryMetadataBuilder>::build_metadata(
                            ::alloc::vec::Vec::new(),
                            &mut entries,
                        );
                    }
                    {
                        <Foo<
                            T,
                        > as frame_support::storage::StorageEntryMetadataBuilder>::build_metadata(
                            ::alloc::vec::Vec::new(),
                            &mut entries,
                        );
                    }
                    {
                        <CountedMap<
                            T,
                        > as frame_support::storage::StorageEntryMetadataBuilder>::build_metadata(
                            ::alloc::vec::Vec::new(),
                            &mut entries,
                        );
                    }
                    entries
                },
            }
        }
    }
    #[doc(hidden)]
    pub(super) struct _GeneratedPrefixForStorageDummy<T>(
        core::marker::PhantomData<(T,)>,
    );
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageDummy<T> {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
                .expect(
                    "No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        fn pallet_prefix_hash() -> [u8; 16] {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name_hash::<
                Pallet<T>,
            >()
                .expect(
                    "No name_hash found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        const STORAGE_PREFIX: &'static str = "Dummy";
        fn storage_prefix_hash() -> [u8; 16] {
            [
                224u8,
                14u8,
                98u8,
                126u8,
                108u8,
                115u8,
                149u8,
                234u8,
                65u8,
                78u8,
                246u8,
                113u8,
                178u8,
                45u8,
                188u8,
                56u8,
            ]
        }
    }
    #[doc(hidden)]
    pub(super) struct _GeneratedPrefixForStorageBar<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageBar<T> {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
                .expect(
                    "No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        fn pallet_prefix_hash() -> [u8; 16] {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name_hash::<
                Pallet<T>,
            >()
                .expect(
                    "No name_hash found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        const STORAGE_PREFIX: &'static str = "Bar";
        fn storage_prefix_hash() -> [u8; 16] {
            [
                23u8,
                65u8,
                91u8,
                65u8,
                140u8,
                169u8,
                212u8,
                251u8,
                133u8,
                16u8,
                151u8,
                202u8,
                205u8,
                108u8,
                17u8,
                216u8,
            ]
        }
    }
    #[doc(hidden)]
    pub(super) struct _GeneratedPrefixForStorageFoo<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageFoo<T> {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
                .expect(
                    "No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        fn pallet_prefix_hash() -> [u8; 16] {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name_hash::<
                Pallet<T>,
            >()
                .expect(
                    "No name_hash found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        const STORAGE_PREFIX: &'static str = "Foo";
        fn storage_prefix_hash() -> [u8; 16] {
            [
                108u8,
                243u8,
                64u8,
                109u8,
                251u8,
                54u8,
                109u8,
                212u8,
                159u8,
                205u8,
                80u8,
                58u8,
                233u8,
                100u8,
                125u8,
                228u8,
            ]
        }
    }
    #[doc(hidden)]
    pub struct _GeneratedCounterPrefixForStorageCountedMap<T>(
        core::marker::PhantomData<(T,)>,
    );
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedCounterPrefixForStorageCountedMap<T> {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
                .expect(
                    "No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        fn pallet_prefix_hash() -> [u8; 16] {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name_hash::<
                Pallet<T>,
            >()
                .expect(
                    "No name_hash found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        const STORAGE_PREFIX: &'static str = "CounterForCountedMap";
        fn storage_prefix_hash() -> [u8; 16] {
            [
                26u8,
                253u8,
                206u8,
                250u8,
                233u8,
                88u8,
                152u8,
                5u8,
                155u8,
                244u8,
                16u8,
                8u8,
                2u8,
                251u8,
                89u8,
                53u8,
            ]
        }
    }
    impl<T: Config> frame_support::storage::types::CountedStorageMapInstance
    for _GeneratedPrefixForStorageCountedMap<T> {
        type CounterPrefix = _GeneratedCounterPrefixForStorageCountedMap<T>;
    }
    #[doc(hidden)]
    pub struct _GeneratedPrefixForStorageCountedMap<T>(core::marker::PhantomData<(T,)>);
    impl<T: Config> frame_support::traits::StorageInstance
    for _GeneratedPrefixForStorageCountedMap<T> {
        fn pallet_prefix() -> &'static str {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                Pallet<T>,
            >()
                .expect(
                    "No name found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        fn pallet_prefix_hash() -> [u8; 16] {
            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name_hash::<
                Pallet<T>,
            >()
                .expect(
                    "No name_hash found for the pallet in the runtime! This usually means that the pallet wasn't added to `construct_runtime!`.",
                )
        }
        const STORAGE_PREFIX: &'static str = "CountedMap";
        fn storage_prefix_hash() -> [u8; 16] {
            [
                140u8,
                2u8,
                22u8,
                43u8,
                54u8,
                95u8,
                1u8,
                128u8,
                154u8,
                149u8,
                167u8,
                242u8,
                20u8,
                127u8,
                151u8,
                180u8,
            ]
        }
    }
    #[doc(hidden)]
    pub mod __substrate_inherent_check {
        #[doc(hidden)]
        pub use __is_inherent_part_defined_3 as is_inherent_part_defined;
    }
    /// Hidden instance generated to be internally used when module is used without
    /// instance.
    #[doc(hidden)]
    pub type __InherentHiddenInstance = ();
    impl<
        T: Config,
    > frame_support::traits::OnFinalize<frame_system::pallet_prelude::BlockNumberFor<T>>
    for Pallet<T> {
        fn on_finalize(n: frame_system::pallet_prelude::BlockNumberFor<T>) {
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "on_finalize",
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ::tracing::Level::TRACE,
                            ::core::option::Option::Some(
                                "inconsistent-weight-name/vulnerable/vulnerable-1/src/lib.rs",
                            ),
                            ::core::option::Option::Some(61u32),
                            ::core::option::Option::Some(
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&__CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE
                        <= ::tracing::level_filters::LevelFilter::current()
                    && {
                        interest = __CALLSITE.interest();
                        !interest.is_never()
                    }
                    && ::tracing::__macro_support::__is_enabled(
                        __CALLSITE.metadata(),
                        interest,
                    )
                {
                    let meta = __CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(
                        __CALLSITE.metadata(),
                    );
                    if match ::tracing::Level::TRACE {
                        ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                        ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                        ::tracing::Level::INFO => ::tracing::log::Level::Info,
                        ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                        _ => ::tracing::log::Level::Trace,
                    } <= ::tracing::log::STATIC_MAX_LEVEL
                    {
                        if !::tracing::dispatcher::has_been_set() {
                            {
                                span.record_all(
                                    &{ __CALLSITE.metadata().fields().value_set(&[]) },
                                );
                            }
                        } else {
                            {}
                        }
                    } else {
                        {}
                    };
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            <Self as frame_support::traits::Hooks<
                frame_system::pallet_prelude::BlockNumberFor<T>,
            >>::on_finalize(n)
        }
    }
    impl<
        T: Config,
    > frame_support::traits::OnIdle<frame_system::pallet_prelude::BlockNumberFor<T>>
    for Pallet<T> {
        fn on_idle(
            n: frame_system::pallet_prelude::BlockNumberFor<T>,
            remaining_weight: frame_support::weights::Weight,
        ) -> frame_support::weights::Weight {
            <Self as frame_support::traits::Hooks<
                frame_system::pallet_prelude::BlockNumberFor<T>,
            >>::on_idle(n, remaining_weight)
        }
    }
    impl<
        T: Config,
    > frame_support::traits::OnPoll<frame_system::pallet_prelude::BlockNumberFor<T>>
    for Pallet<T> {
        fn on_poll(
            n: frame_system::pallet_prelude::BlockNumberFor<T>,
            weight: &mut frame_support::weights::WeightMeter,
        ) {
            <Self as frame_support::traits::Hooks<
                frame_system::pallet_prelude::BlockNumberFor<T>,
            >>::on_poll(n, weight);
        }
    }
    impl<
        T: Config,
    > frame_support::traits::OnInitialize<
        frame_system::pallet_prelude::BlockNumberFor<T>,
    > for Pallet<T> {
        fn on_initialize(
            n: frame_system::pallet_prelude::BlockNumberFor<T>,
        ) -> frame_support::weights::Weight {
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "on_initialize",
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ::tracing::Level::TRACE,
                            ::core::option::Option::Some(
                                "inconsistent-weight-name/vulnerable/vulnerable-1/src/lib.rs",
                            ),
                            ::core::option::Option::Some(61u32),
                            ::core::option::Option::Some(
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&__CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE
                        <= ::tracing::level_filters::LevelFilter::current()
                    && {
                        interest = __CALLSITE.interest();
                        !interest.is_never()
                    }
                    && ::tracing::__macro_support::__is_enabled(
                        __CALLSITE.metadata(),
                        interest,
                    )
                {
                    let meta = __CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(
                        __CALLSITE.metadata(),
                    );
                    if match ::tracing::Level::TRACE {
                        ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                        ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                        ::tracing::Level::INFO => ::tracing::log::Level::Info,
                        ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                        _ => ::tracing::log::Level::Trace,
                    } <= ::tracing::log::STATIC_MAX_LEVEL
                    {
                        if !::tracing::dispatcher::has_been_set() {
                            {
                                span.record_all(
                                    &{ __CALLSITE.metadata().fields().value_set(&[]) },
                                );
                            }
                        } else {
                            {}
                        }
                    } else {
                        {}
                    };
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            <Self as frame_support::traits::Hooks<
                frame_system::pallet_prelude::BlockNumberFor<T>,
            >>::on_initialize(n)
        }
    }
    impl<T: Config> frame_support::traits::BeforeAllRuntimeMigrations for Pallet<T> {
        fn before_all_runtime_migrations() -> frame_support::weights::Weight {
            use frame_support::traits::{Get, PalletInfoAccess};
            use frame_support::__private::hashing::twox_128;
            use frame_support::storage::unhashed::contains_prefixed_key;
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "before_all",
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ::tracing::Level::TRACE,
                            ::core::option::Option::Some(
                                "inconsistent-weight-name/vulnerable/vulnerable-1/src/lib.rs",
                            ),
                            ::core::option::Option::Some(61u32),
                            ::core::option::Option::Some(
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&__CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE
                        <= ::tracing::level_filters::LevelFilter::current()
                    && {
                        interest = __CALLSITE.interest();
                        !interest.is_never()
                    }
                    && ::tracing::__macro_support::__is_enabled(
                        __CALLSITE.metadata(),
                        interest,
                    )
                {
                    let meta = __CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(
                        __CALLSITE.metadata(),
                    );
                    if match ::tracing::Level::TRACE {
                        ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                        ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                        ::tracing::Level::INFO => ::tracing::log::Level::Info,
                        ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                        _ => ::tracing::log::Level::Trace,
                    } <= ::tracing::log::STATIC_MAX_LEVEL
                    {
                        if !::tracing::dispatcher::has_been_set() {
                            {
                                span.record_all(
                                    &{ __CALLSITE.metadata().fields().value_set(&[]) },
                                );
                            }
                        } else {
                            {}
                        }
                    } else {
                        {}
                    };
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            let pallet_hashed_prefix = <Self as PalletInfoAccess>::name_hash();
            let exists = contains_prefixed_key(&pallet_hashed_prefix);
            if !exists {
                let default_version = frame_support::traits::StorageVersion::new(0);
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api::log(
                            format_args!(
                                "🐥 New pallet {0:?} detected in the runtime. The pallet has no defined storage version, so the on-chain version is being initialized to {1:?}.",
                                <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                                    Self,
                                >()
                                    .unwrap_or("<unknown pallet name>"),
                                default_version,
                            ),
                            lvl,
                            &(
                                frame_support::LOG_TARGET,
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                                ::log::__private_api::loc(),
                            ),
                            (),
                        );
                    }
                };
                default_version.put::<Self>();
                <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
            } else {
                <T as frame_system::Config>::DbWeight::get().reads(1)
            }
        }
    }
    impl<T: Config> frame_support::traits::OnRuntimeUpgrade for Pallet<T> {
        fn on_runtime_upgrade() -> frame_support::weights::Weight {
            let __within_span__ = {
                use ::tracing::__macro_support::Callsite as _;
                static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                    static META: ::tracing::Metadata<'static> = {
                        ::tracing_core::metadata::Metadata::new(
                            "on_runtime_update",
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ::tracing::Level::TRACE,
                            ::core::option::Option::Some(
                                "inconsistent-weight-name/vulnerable/vulnerable-1/src/lib.rs",
                            ),
                            ::core::option::Option::Some(61u32),
                            ::core::option::Option::Some(
                                "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ),
                            ::tracing_core::field::FieldSet::new(
                                &[],
                                ::tracing_core::callsite::Identifier(&__CALLSITE),
                            ),
                            ::tracing::metadata::Kind::SPAN,
                        )
                    };
                    ::tracing::callsite::DefaultCallsite::new(&META)
                };
                let mut interest = ::tracing::subscriber::Interest::never();
                if ::tracing::Level::TRACE <= ::tracing::level_filters::STATIC_MAX_LEVEL
                    && ::tracing::Level::TRACE
                        <= ::tracing::level_filters::LevelFilter::current()
                    && {
                        interest = __CALLSITE.interest();
                        !interest.is_never()
                    }
                    && ::tracing::__macro_support::__is_enabled(
                        __CALLSITE.metadata(),
                        interest,
                    )
                {
                    let meta = __CALLSITE.metadata();
                    ::tracing::Span::new(meta, &{ meta.fields().value_set(&[]) })
                } else {
                    let span = ::tracing::__macro_support::__disabled_span(
                        __CALLSITE.metadata(),
                    );
                    if match ::tracing::Level::TRACE {
                        ::tracing::Level::ERROR => ::tracing::log::Level::Error,
                        ::tracing::Level::WARN => ::tracing::log::Level::Warn,
                        ::tracing::Level::INFO => ::tracing::log::Level::Info,
                        ::tracing::Level::DEBUG => ::tracing::log::Level::Debug,
                        _ => ::tracing::log::Level::Trace,
                    } <= ::tracing::log::STATIC_MAX_LEVEL
                    {
                        if !::tracing::dispatcher::has_been_set() {
                            {
                                span.record_all(
                                    &{ __CALLSITE.metadata().fields().value_set(&[]) },
                                );
                            }
                        } else {
                            {}
                        }
                    } else {
                        {}
                    };
                    span
                }
            };
            let __tracing_guard__ = __within_span__.enter();
            {
                let lvl = ::log::Level::Debug;
                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                    ::log::__private_api::log(
                        format_args!(
                            "✅ no migration for {0}",
                            <<T as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::name::<
                                Self,
                            >()
                                .unwrap_or("<unknown pallet name>"),
                        ),
                        lvl,
                        &(
                            frame_support::LOG_TARGET,
                            "pallet_inconsistent_weight_name_vulnerable_1::pallet",
                            ::log::__private_api::loc(),
                        ),
                        (),
                    );
                }
            };
            <Self as frame_support::traits::Hooks<
                frame_system::pallet_prelude::BlockNumberFor<T>,
            >>::on_runtime_upgrade()
        }
    }
    impl<
        T: Config,
    > frame_support::traits::OffchainWorker<
        frame_system::pallet_prelude::BlockNumberFor<T>,
    > for Pallet<T> {
        fn offchain_worker(n: frame_system::pallet_prelude::BlockNumberFor<T>) {
            <Self as frame_support::traits::Hooks<
                frame_system::pallet_prelude::BlockNumberFor<T>,
            >>::offchain_worker(n)
        }
    }
    impl<T: Config> frame_support::traits::IntegrityTest for Pallet<T> {
        fn integrity_test() {
            frame_support::__private::sp_io::TestExternalities::default()
                .execute_with(|| {
                    <Self as frame_support::traits::Hooks<
                        frame_system::pallet_prelude::BlockNumberFor<T>,
                    >>::integrity_test()
                });
        }
    }
    #[cfg(feature = "std")]
    impl<T: Config> frame_support::sp_runtime::BuildStorage for GenesisConfig<T> {
        fn assimilate_storage(
            &self,
            storage: &mut frame_support::sp_runtime::Storage,
        ) -> std::result::Result<(), std::string::String> {
            frame_support::__private::BasicExternalities::execute_with_storage(
                storage,
                || {
                    self.build();
                    Ok(())
                },
            )
        }
    }
    #[doc(hidden)]
    pub mod __substrate_genesis_config_check {
        #[doc(hidden)]
        pub use __is_genesis_config_defined_4 as is_genesis_config_defined;
        #[doc(hidden)]
        pub use __is_std_macro_defined_for_genesis_4 as is_std_enabled_for_genesis;
    }
    #[doc(hidden)]
    pub mod __substrate_origin_check {
        #[doc(hidden)]
        pub use __is_origin_part_defined_5 as is_origin_part_defined;
    }
    #[doc(hidden)]
    pub mod __substrate_validate_unsigned_check {
        #[doc(hidden)]
        pub use __is_validate_unsigned_part_defined_6 as is_validate_unsigned_part_defined;
    }
    pub use __tt_default_parts_7 as tt_default_parts;
    pub use __tt_extra_parts_7 as tt_extra_parts;
    pub use __tt_default_parts_v2_7 as tt_default_parts_v2;
}
impl<T: Config> Pallet<T> {
    #[allow(dead_code)]
    fn accumulate_foo(
        origin: T::RuntimeOrigin,
        increase_by: T::Balance,
    ) -> DispatchResult {
        let _sender = ensure_signed(origin)?;
        let prev = Foo::<T>::get();
        let result = Foo::<
            T,
        >::mutate(|x| {
            *x = x.saturating_add(increase_by);
            *x
        });
        if !(prev + increase_by == result) {
            ::core::panicking::panic("assertion failed: prev + increase_by == result")
        }
        Ok(())
    }
}
#[scale_info(skip_type_params(T))]
pub struct WatchDummy<T: Config + Send + Sync>(PhantomData<T>);
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl<T: Config + Send + Sync> ::codec::Encode for WatchDummy<T>
    where
        PhantomData<T>: ::codec::Encode,
        PhantomData<T>: ::codec::Encode,
    {
        fn size_hint(&self) -> usize {
            ::codec::Encode::size_hint(&&self.0)
        }
        fn encode_to<__CodecOutputEdqy: ::codec::Output + ?::core::marker::Sized>(
            &self,
            __codec_dest_edqy: &mut __CodecOutputEdqy,
        ) {
            ::codec::Encode::encode_to(&&self.0, __codec_dest_edqy)
        }
        fn encode(&self) -> ::codec::alloc::vec::Vec<::core::primitive::u8> {
            ::codec::Encode::encode(&&self.0)
        }
        fn using_encoded<
            __CodecOutputReturn,
            __CodecUsingEncodedCallback: ::core::ops::FnOnce(
                    &[::core::primitive::u8],
                ) -> __CodecOutputReturn,
        >(&self, f: __CodecUsingEncodedCallback) -> __CodecOutputReturn {
            ::codec::Encode::using_encoded(&&self.0, f)
        }
    }
    #[automatically_derived]
    impl<T: Config + Send + Sync> ::codec::EncodeLike for WatchDummy<T>
    where
        PhantomData<T>: ::codec::Encode,
        PhantomData<T>: ::codec::Encode,
    {}
};
#[allow(deprecated)]
const _: () = {
    #[automatically_derived]
    impl<T: Config + Send + Sync> ::codec::Decode for WatchDummy<T>
    where
        PhantomData<T>: ::codec::Decode,
        PhantomData<T>: ::codec::Decode,
    {
        fn decode<__CodecInputEdqy: ::codec::Input>(
            __codec_input_edqy: &mut __CodecInputEdqy,
        ) -> ::core::result::Result<Self, ::codec::Error> {
            ::core::result::Result::Ok(
                WatchDummy::<
                    T,
                >({
                    let __codec_res_edqy = <PhantomData<
                        T,
                    > as ::codec::Decode>::decode(__codec_input_edqy);
                    match __codec_res_edqy {
                        ::core::result::Result::Err(e) => {
                            return ::core::result::Result::Err(
                                e.chain("Could not decode `WatchDummy.0`"),
                            );
                        }
                        ::core::result::Result::Ok(__codec_res_edqy) => __codec_res_edqy,
                    }
                }),
            )
        }
    }
};
#[automatically_derived]
impl<T: ::core::clone::Clone + Config + Send + Sync> ::core::clone::Clone
for WatchDummy<T> {
    #[inline]
    fn clone(&self) -> WatchDummy<T> {
        WatchDummy(::core::clone::Clone::clone(&self.0))
    }
}
#[automatically_derived]
impl<T: ::core::cmp::Eq + Config + Send + Sync> ::core::cmp::Eq for WatchDummy<T> {
    #[inline]
    #[doc(hidden)]
    #[coverage(off)]
    fn assert_receiver_is_total_eq(&self) -> () {
        let _: ::core::cmp::AssertParamIsEq<PhantomData<T>>;
    }
}
#[automatically_derived]
impl<T: Config + Send + Sync> ::core::marker::StructuralPartialEq for WatchDummy<T> {}
#[automatically_derived]
impl<T: ::core::cmp::PartialEq + Config + Send + Sync> ::core::cmp::PartialEq
for WatchDummy<T> {
    #[inline]
    fn eq(&self, other: &WatchDummy<T>) -> bool {
        self.0 == other.0
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    impl<T: Config + Send + Sync> ::scale_info::TypeInfo for WatchDummy<T>
    where
        PhantomData<T>: ::scale_info::TypeInfo + 'static,
        T: Config + Send + Sync + 'static,
    {
        type Identity = Self;
        fn type_info() -> ::scale_info::Type {
            ::scale_info::Type::builder()
                .path(
                    ::scale_info::Path::new_with_replace(
                        "WatchDummy",
                        "pallet_inconsistent_weight_name_vulnerable_1",
                        &[],
                    ),
                )
                .type_params(
                    <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            ::scale_info::TypeParameter::new(
                                "T",
                                ::core::option::Option::None,
                            ),
                        ]),
                    ),
                )
                .composite(
                    ::scale_info::build::Fields::unnamed()
                        .field(|f| f.ty::<PhantomData<T>>().type_name("PhantomData<T>")),
                )
        }
    }
};
impl<T: Config + Send + Sync> core::fmt::Debug for WatchDummy<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_fmt(format_args!("WatchDummy"))
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
