#[derive(PartialEq, Debug)]
pub enum Error {
    OutOfBounds {
        i: usize,
        len: usize,
    },
    MissingLengthInformation,
    ExcessBits,
    InvalidByteCount {
        given: usize,
        expected: usize,
    },
}