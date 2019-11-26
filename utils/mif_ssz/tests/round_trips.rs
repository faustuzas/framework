use ssz::{Encode, Decode};
use ssz_derive::{Encode, Decode};
use core::num::NonZeroUsize;
use ethereum_types::{H256, U128, U256};

mod round_trips {
    use super::*;

    fn round_trip<T: Encode + Decode + std::fmt::Debug + PartialEq>(item: T) {
        println!("{:?}", item);
        println!("{:?}", &item.as_ssz_bytes());
        assert_eq!(T::from_ssz_bytes(&item.as_ssz_bytes()).unwrap(), item)
    }
    
    #[test]
    fn test_bool() {
        round_trip(true);
        round_trip(false);
    }

    #[test]
    fn test_u8() {
        round_trip(u8::min_value());
        round_trip(10 as u8);
        round_trip(u8::max_value());
    }

    #[test]
    fn test_u16() {
        round_trip(u16::min_value());
        round_trip(100 as u16);
        round_trip(u16::max_value());
    }

    #[test]
    fn test_u32() {
        round_trip(u32::min_value());
        round_trip(1000 as u32);
        round_trip(u32::max_value());
    }

    #[test]
    fn test_u64() {
        round_trip(u64::min_value());
        round_trip(10000 as u64);
        round_trip(u64::max_value());
    }

    #[test]
    fn test_usize() {
        round_trip(usize::min_value());
        round_trip(usize::max_value());
    }

    #[test]
    fn test_vec() {
        let vec: Vec<u32> = vec![];
        round_trip(vec);

        let vec: Vec<u32> = vec![0, 1, 2, 2, 4, 5, 6, 7, 8, 9, 10];
        round_trip(vec);

        let vec: Vec<Vec<Vec<Vec<Vec<Vec<Vec<u32>>>>>>> = vec![vec![vec![vec![vec![vec![vec![], vec![1], vec![1, 2], vec![1, 2, 3], vec![1, 2, 3, 4, 5]]]]]]];
        round_trip(vec);
    }

    #[test]
    fn test_union() {
        round_trip(Some(usize::max_value()));
        round_trip(Some(vec![usize::max_value(), usize::max_value()]));
        round_trip(Some(Some(vec![usize::max_value(), usize::max_value()])));
        round_trip(None as Option<usize>);
    }

    #[test]
    fn test_non_zero_usize() {
        round_trip(NonZeroUsize::new(usize::max_value()).unwrap());
        round_trip(NonZeroUsize::new(usize::min_value() + 1).unwrap());
    }

    #[test]
    fn test_h256() {
        round_trip(H256::zero());
        round_trip(H256::from_slice(&[42; 32]));
    }

    #[test]
    fn test_u128() {
        round_trip(U128::zero());
        round_trip(U128::one());
        round_trip(U128::MAX);
    }

    #[test]
    fn test_u256() {
        round_trip(U256::zero());
        round_trip(U256::one());
        round_trip(U256::MAX);
    }

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct FixedStruct {
        a: usize,
        b: u8,
        c: U128,
        d: Option<u16>
    }

    #[test]
    fn test_fixed_struct_derive() {
        round_trip(FixedStruct {
            a: 0,
            b: 0,
            c: U128::from_dec_str("0").unwrap(),
            d: None
        });

        round_trip(FixedStruct {
            a: 500,
            b: 15,
            c: U128::from_dec_str("123456").unwrap(),
            d: Some(500)
        })
    }

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct VariableStruct {
        a: Vec<Vec<Vec<Option<Vec<u32>>>>>,
        b: Vec<u8>
    }

    #[test]
    fn test_variable_struct_derive() {
        round_trip(VariableStruct {
            a: vec![vec![vec![
                Some(vec![5, 6, 7]),
                Some(vec![128, 128, 256]),
                None,
                Some(vec![33]),
                None,
                None,
                Some(vec![])
            ]]],
            b: vec![]
        });
    }

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct IgnoreFieldsStruct {
        a: usize,

        #[ssz(skip_serializing)]
        #[ssz(skip_deserializing)]
        b: u64
    }

    #[test]
    fn test_ignore_fields_struct_derive() {
        let original = IgnoreFieldsStruct {
            a: 500,
            b: 1000
        };

        let after_round_trip = IgnoreFieldsStruct::from_ssz_bytes(&original.as_ssz_bytes()).unwrap();

        assert_eq!(after_round_trip, IgnoreFieldsStruct {
           a: original.a,
           b: Default::default()
        });
    }
}