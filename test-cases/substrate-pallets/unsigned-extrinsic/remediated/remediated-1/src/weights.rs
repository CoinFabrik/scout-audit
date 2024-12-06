#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

pub trait WeightInfo {
	fn safe_call() -> Weight;
}

pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	fn safe_call() -> Weight {
		Weight::from_parts(19_000_000_u64, 0)
	}
}

impl WeightInfo for () {
	fn safe_call() -> Weight {
		Weight::from_parts(19_000_000_u64, 0)
	}
}
