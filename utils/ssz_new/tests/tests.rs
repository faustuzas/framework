use ssz::Serialize;
use ssz_derive::SszSerialize;

mod serialize_derive {
    use crate::*;

    #[derive(SszSerialize)]
    struct Fixed {
        a: u16,
        b: bool
    }

    #[test]
    fn serialize_fixed_struct() {
        let fixed = Fixed {
            a: 22,
            b: true
        };

        assert_eq!(fixed.serialize().unwrap(), vec![22, 0, 1])
    }

    #[derive(SszSerialize)]
    struct Variable {
        a: u16,
        b: Vec<u8>,
        c: bool
    }

    #[test]
    fn serialize_variable_struct() {
        let variable = Variable {
            a: u16::max_value(),
            b: vec![1, 2, 3, 4, 5],
            c: false
        };

        assert_eq!(variable.serialize().unwrap(),
                   vec![u8::max_value(), u8::max_value(), 7, 0, 0, 0, 0, 1, 2, 3, 4, 5])
    }
}