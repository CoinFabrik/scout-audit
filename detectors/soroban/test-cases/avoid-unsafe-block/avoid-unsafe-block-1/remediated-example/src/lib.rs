#![no_std]
use soroban_sdk::{contract, contractimpl};
#[contract]
pub struct AvoidUnsafeBlock;
#[contractimpl]
impl AvoidUnsafeBlock {
    pub fn unsafe_function(n: u64) -> u64 {
        let mut i = n as f64;
        let mut y = i.to_bits();
        y = 0x5fe6ec85e7de30da - (y >> 1);
        i = f64::from_bits(y);
        i *= 1.5 - 0.5 * n as f64 * i * i;
        i *= 1.5 - 0.5 * n as f64 * i * i;
        i.to_bits()
    }
}
#[cfg(test)]
mod tests {
    use crate::AvoidUnsafeBlock;
    #[test]
    fn test_unsafe_block() {
        let test_value = 8;
        let result = AvoidUnsafeBlock::unsafe_function(test_value);
        let inverse = inverse_square_root_without_unsafe(test_value);

        assert_eq!((inverse - result) / inverse, 0);
        assert_eq!((inverse - result) / result, 0);
    }

    fn inverse_square_root_without_unsafe(n: u64) -> u64 {
        (1.0 / (n as f64).sqrt()).to_bits()
    }
}
