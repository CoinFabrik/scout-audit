#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn another_dummy_call() -> Weight;
	fn dummy_call(x: u64) -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn another_dummy_call() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
	}
	fn dummy_call(x: u64) -> Weight {
		Weight::from_parts(10_000_000_u64 * x, 0)
	}
}

impl WeightInfo for () {
	fn another_dummy_call() -> Weight {
		Weight::from_parts(10_000_000_u64, 0)
	}
	fn dummy_call(x: u64) -> Weight {
		Weight::from_parts(10_000_000_u64 * x, 0)
	}
}
