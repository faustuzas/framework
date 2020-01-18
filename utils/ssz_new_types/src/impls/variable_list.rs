use super::*;
use ssz::*;

impl<T: Encode + Clone, N: Unsigned> Encode for VariableList<T, N> {
    fn as_ssz_bytes(&self) -> Vec<u8> {
        self.to_vec().as_ssz_bytes()
    }

    fn is_ssz_fixed_len() -> bool {
        <Vec<T>>::is_ssz_fixed_len()
    }
}

impl<T: Decode, N: Unsigned> Decode for VariableList<T, N> {
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        let items = <Vec<T>>::from_ssz_bytes(bytes)?;

        Self::new(items).map_err(|e| {
            DecodeError::BytesInvalid(format!("Failed while creating VariableList: {:?}", e))
        })
    }

    fn is_ssz_fixed_len() -> bool {
        <Vec<T>>::is_ssz_fixed_len()
    }

    fn ssz_fixed_len() -> usize {
        <Vec<T>>::ssz_fixed_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;

    mod serialize {
        use super::*;

        #[test]
        fn fixed() {
            let vec = <VariableList<u16, U4>>::new(vec![1, 2, 3, 4]).unwrap();
            assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0, 4, 0]);
        }
    }
}
