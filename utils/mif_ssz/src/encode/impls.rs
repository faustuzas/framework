use super::*;

impl Encode for bool {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn ssz_append(&self, buf: &mut Vec<u8>) {
        let value: u8 = if *self { 1 } else { 0 };
        buf.extend_from_slice(&value.to_le_bytes())
    }

    fn ssz_fixed_len() -> usize {
        1
    }

    fn ssz_bytes_len(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_bool() {
        let val = true;
        assert_eq!(val.as_ssz_bytes(), vec![1]);

        let val = false;
        assert_eq!(val.as_ssz_bytes(), vec![0]);
    }
}