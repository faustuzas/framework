use super::*;
use ssz::*;

impl<T: Encode + Clone, N: Unsigned> Encode for VariableList<T, N> {
    fn ssz_append(&self, buf: &mut Vec<u8>) {
        buf.append(&mut self.to_vec().as_ssz_bytes())
    }

    fn is_ssz_fixed_len() -> bool {
        T::is_ssz_fixed_len()
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

    #[test]
    fn encode() {
        let vec = <VariableList<u16, U4>>::new(vec![1, 2, 3, 4]).expect("Test");
        assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0, 3, 0, 4, 0]);

        let vec = <VariableList<u16, U20>>::new(vec![1, 2]).expect("Test");
        assert_eq!(vec.as_ssz_bytes(), vec![1, 0, 2, 0]);
    }

    #[test]
    fn decode() {
        let list = <VariableList<u16, U3>>::from_ssz_bytes(&[1, 0, 2, 0, 3, 0]).expect("Test");
        assert_eq!(list.to_vec(), &vec![1_u16, 2_u16, 3_u16]);

        let list = <VariableList<u16, U1024>>::from_ssz_bytes(&[1, 0, 2, 0, 3, 0]).expect("Test");
        assert_eq!(list.to_vec(), &vec![1_u16, 2_u16, 3_u16]);

        assert!(<VariableList<u8, U1>>::from_ssz_bytes(&[1, 2, 3]).is_err())
    }
}
