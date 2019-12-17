use super::*;
use ssz::Error;

impl<N: Unsigned + Clone> ssz::Serialize for Bitfield<length::Variable<N>> {
    fn serialize(&self) -> Result<Vec<u8>, ssz::Error> {
        Ok(self.clone().into_bytes())
    }

    fn is_variable_size() -> bool {
        true
    }
}

impl<N: Unsigned + Clone> ssz::Deserialize for Bitfield<length::Variable<N>> {
    fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        Self::from_bytes(bytes.to_vec()).map_err(|e| {
            ssz::Error::InvalidBytes(format!("Failed while creating BitList: {:?}", e))
        })
    }

    fn is_variable_size() -> bool {
        true
    }

    fn fixed_length() -> usize {
        std::cmp::max(1, (N::to_usize() + 7) / 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssz::{Deserialize, Serialize};
    use typenum::*;

    mod bitlist {
        use super::*;

        pub type BitList0 = Bitfield<length::Variable<U0>>;
        pub type BitList1 = Bitfield<length::Variable<U1>>;
        pub type BitList8 = Bitfield<length::Variable<U8>>;
        pub type BitList16 = Bitfield<length::Variable<U16>>;
        pub type BitList1024 = Bitfield<length::Variable<U1024>>;

        #[test]
        fn serialize() {
            assert_eq!(
                BitList0::with_capacity(0).unwrap().serialize().unwrap(),
                vec![0b0000_00001],
            );

            assert_eq!(
                BitList1::with_capacity(0).unwrap().serialize().unwrap(),
                vec![0b0000_00001],
            );

            assert_eq!(
                BitList1::with_capacity(1).unwrap().serialize().unwrap(),
                vec![0b0000_00010],
            );

            assert_eq!(
                BitList8::with_capacity(8).unwrap().serialize().unwrap(),
                vec![0b0000_0000, 0b0000_0001],
            );

            assert_eq!(
                BitList8::with_capacity(7).unwrap().serialize().unwrap(),
                vec![0b1000_0000]
            );

            let mut b = BitList8::with_capacity(8).unwrap();
            for i in 0..8 {
                b.set(i, true).unwrap();
            }
            assert_eq!(b.serialize().unwrap(), vec![255, 0b0000_0001]);

            let mut b = BitList8::with_capacity(8).unwrap();
            for i in 0..4 {
                b.set(i, true).unwrap();
            }
            assert_eq!(b.serialize().unwrap(), vec![0b0000_1111, 0b0000_0001]);

            assert_eq!(
                BitList16::with_capacity(16).unwrap().serialize().unwrap(),
                vec![0b0000_0000, 0b0000_0000, 0b0000_0001]
            );
        }
    }
}
