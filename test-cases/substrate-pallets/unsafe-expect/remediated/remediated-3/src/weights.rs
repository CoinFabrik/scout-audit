#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn unsafe_get_storage() -> Weight;
	fn set_storage() -> Weight;
	fn clear_storage() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn unsafe_get_storage() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
	}
	fn set_storage() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads_writes(1_u64, 1_u64))
	}
	fn clear_storage() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads_writes(1_u64, 1_u64))
	}
}

impl WeightInfo for () {
	fn unsafe_get_storage() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
	}
	fn set_storage() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads_writes(1_u64, 1_u64))
	}
	fn clear_storage() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads_writes(1_u64, 1_u64))
	}
}
