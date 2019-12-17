use super::*;
use ssz::{Deserialize, Error, Serialize};

impl<T: Serialize + Clone, N: Unsigned> ssz::Serialize for VariableList<T, N> {
    fn serialize(&self) -> Result<Vec<u8>, Error> {
        self.vec.serialize()
    }

    fn is_variable_size() -> bool {
        <Vec<T>>::is_variable_size()
    }
}

impl<T: Deserialize, N: Unsigned> ssz::Deserialize for VariableList<T, N> {
    fn deserialize(bytes: &[u8]) -> Result<Self, Error> {
        let items = <Vec<T>>::deserialize(bytes)?;

        if items.len() <= N::to_usize() {
            Self::new(items).map_err(|e| {
                ssz::Error::InvalidBytes(format!("Failed while creating VariableList: {:?}", e))
            })
        } else {
            Err(ssz::Error::TooMuchElements {
                got: items.len(),
                max: N::to_usize(),
            })
        }
    }

    fn is_variable_size() -> bool {
        <Vec<T>>::is_variable_size()
    }

    fn fixed_length() -> usize {
        <Vec<T>>::fixed_length()
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
            assert_eq!(vec.serialize().unwrap(), vec![1, 0, 2, 0, 3, 0, 4, 0]);
        }
    }
}
