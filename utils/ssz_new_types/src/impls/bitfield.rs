use super::*;
use ssz::*;

impl<N: Unsigned + Clone> Encode for Bitfield<length::Variable<N>> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.clone().into_bytes()
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }
}

impl<N: Unsigned + Clone> Decode for Bitfield<length::Variable<N>> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(bytes.to_vec()).map_err(|e| {
            DecodeError::BytesInvalid(format!("Failed while creating BitList: {:?}", e))
        })
    }

    fn is_ssz_fixed_len() -> bool {
        false
    }

    fn ssz_fixed_len() -> usize {
        std::cmp::max(1, (N::to_usize() + 7) / 8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;

    mod bitlist {
        use super::*;

        pub type BitList0 = Bitfield<length::Variable<U0>>;
        pub type BitList1 = Bitfield<length::Variable<U1>>;
        pub type BitList8 = Bitfield<length::Variable<U8>>;
        pub type BitList16 = Bitfield<length::Variable<U16>>;

        #[test]
        fn serialize() {
            assert_eq!(
                BitList0::with_capacity(0).unwrap().as_ssz_bytes(),
                vec![0b0000_00001],
            );

            assert_eq!(
                BitList1::with_capacity(0).unwrap().as_ssz_bytes(),
                vec![0b0000_00001],
            );

            assert_eq!(
                BitList1::with_capacity(1).unwrap().as_ssz_bytes(),
                vec![0b0000_00010],
            );

            assert_eq!(
                BitList8::with_capacity(8).unwrap().as_ssz_bytes(),
                vec![0b0000_0000, 0b0000_0001],
            );

            assert_eq!(
                BitList8::with_capacity(7).unwrap().as_ssz_bytes(),
                vec![0b1000_0000]
            );

            let mut b = BitList8::with_capacity(8).unwrap();
            for i in 0..8 {
                b.set(i, true).unwrap();
            }
            assert_eq!(b.as_ssz_bytes(), vec![255, 0b0000_0001]);

            let mut b = BitList8::with_capacity(8).unwrap();
            for i in 0..4 {
                b.set(i, true).unwrap();
            }
            assert_eq!(b.as_ssz_bytes(), vec![0b0000_1111, 0b0000_0001]);

            assert_eq!(
                BitList16::with_capacity(16).unwrap().as_ssz_bytes(),
                vec![0b0000_0000, 0b0000_0000, 0b0000_0001]
            );
        }
    }
}
