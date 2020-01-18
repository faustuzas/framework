//use ssz::{Deserialize, Serialize};
//use ssz_derive::{SszDeserialize, SszSerialize};
//
//#[derive(SszSerialize, SszDeserialize, PartialEq, Debug)]
//struct Fixed {
//    a: u16,
//    b: bool,
//}
//
//#[derive(SszSerialize, SszDeserialize, PartialEq, Debug)]
//struct Variable {
//    a: u16,
//    b: Vec<u8>,
//    c: bool,
//}
//
//#[derive(SszSerialize, SszDeserialize, PartialEq, Debug)]
//struct Nested {
//    fixed: Fixed,
//    variable: Variable,
//}
//
//mod serialize_derive {
//    use crate::*;
//
//    #[test]
//    fn is_fixed_size() {
//        assert!(<Nested as Serialize>::is_variable_size());
//        assert!(<Variable as Serialize>::is_variable_size());
//        assert!(!<Fixed as Serialize>::is_variable_size());
//    }
//
//    #[test]
//    fn serialize_fixed_struct() {
//        let fixed = Fixed { a: 22, b: true };
//
//        assert_eq!(fixed.serialize().unwrap(), vec![22, 0, 1])
//    }
//
//    #[test]
//    fn serialize_variable_struct() {
//        let variable = Variable {
//            a: u16::max_value(),
//            b: vec![1, 2, 3, 4, 5],
//            c: false,
//        };
//
//        assert_eq!(
//            variable.serialize().unwrap(),
//            vec![
//                u8::max_value(),
//                u8::max_value(),
//                7,
//                0,
//                0,
//                0,
//                0,
//                1,
//                2,
//                3,
//                4,
//                5
//            ]
//        )
//    }
//
//    #[test]
//    fn serialize_nested_struct() {
//        let nested = Nested {
//            fixed: Fixed { a: 5, b: false },
//            variable: Variable {
//                a: 80,
//                b: vec![1, 2, 3, 4],
//                c: true,
//            },
//        };
//
//        assert_eq!(
//            nested.serialize().unwrap(),
//            vec![5, 0, 0, 7, 0, 0, 0, 80, 0, 7, 0, 0, 0, 1, 1, 2, 3, 4]
//        );
//    }
//}
//
//mod deserialize_derive {
//    use crate::*;
//
//    #[test]
//    fn deserialize_fixed_struct() {
//        let fixed = Fixed { a: 22, b: true };
//
//        assert_eq!(Fixed::deserialize(&[22, 0, 1]).unwrap(), fixed);
//    }
//
//    #[test]
//    fn deserialize_variable_struct() {
//        let variable = Variable {
//            a: u16::max_value(),
//            b: vec![1, 2, 3, 4, 5],
//            c: false,
//        };
//
//        assert_eq!(
//            Variable::deserialize(&[
//                u8::max_value(),
//                u8::max_value(),
//                7,
//                0,
//                0,
//                0,
//                0,
//                1,
//                2,
//                3,
//                4,
//                5
//            ])
//            .unwrap(),
//            variable
//        );
//    }
//
//    #[test]
//    fn deserialize_nested_struct() {
//        let nested = Nested {
//            fixed: Fixed { a: 5, b: false },
//            variable: Variable {
//                a: 80,
//                b: vec![1, 2, 3, 4],
//                c: true,
//            },
//        };
//
//        assert_eq!(
//            Nested::deserialize(&[5, 0, 0, 7, 0, 0, 0, 80, 0, 7, 0, 0, 0, 1, 1, 2, 3, 4]).unwrap(),
//            nested
//        );
//    }
//}
