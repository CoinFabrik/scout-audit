#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod iterators_over_indexing {

    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct IteratorsOverIndexing {
        value: Vec<u8>,
    }

    impl IteratorsOverIndexing {
        #[ink(constructor)]
        pub fn new(value1: u8, value2: u8, value3: u8) -> Self {
            Self {
                value: Vec::from([value1, value2, value3]),
            }
        }

        #[ink(message)]
        pub fn index_bad(&self) -> u32 {
            let mut ret = 0_u32;
            ink::env::debug_println!("this will panic");
            for i in 0..3 {
                ret += u32::from(self.value[i]);
            }
            ret
        }
    }
}
