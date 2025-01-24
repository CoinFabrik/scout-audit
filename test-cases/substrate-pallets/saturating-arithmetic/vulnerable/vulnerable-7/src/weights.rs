#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn set_dummy_benchmark() -> Weight;
	fn decrement_dummy() -> Weight;
	fn sort_vector(x: u32, ) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn set_dummy_benchmark() -> Weight {
		Weight::from_parts(19_000_000_u64, 0)
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn decrement_dummy() -> Weight {
		Weight::from_parts(18_000_000_u64, 0)
			.saturating_add(T::DbWeight::get().reads(1_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	fn sort_vector(x: u32, ) -> Weight {
		Weight::from_parts(0_u64, 0)
			// Standard Error: 2
			.saturating_add(Weight::from_parts(520_u64, 0).saturating_mul(x as u64))
	}
}

impl WeightInfo for () {
	fn set_dummy_benchmark() -> Weight {
		Weight::from_parts(19_000_000_u64, 0)
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn decrement_dummy() -> Weight {
		Weight::from_parts(18_000_000_u64, 0)
			.saturating_add(RocksDbWeight::get().reads(1_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	fn sort_vector(x: u32, ) -> Weight {
		Weight::from_parts(0_u64, 0)
			.saturating_add(Weight::from_parts(520_u64, 0).saturating_mul(x as u64))
	}
}
