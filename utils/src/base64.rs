use base64::{engine::general_purpose::STANDARD, DecodeError, Engine as _};

#[inline]
pub fn b64_encode<T: AsRef<[u8]>>(input: T) -> String {
    STANDARD.encode(input)
}

#[inline]
pub fn b64_decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    STANDARD.decode(input)
}

pub fn b64_encode_old<T: AsRef<[u8]>>(input: T) -> String {
    #[allow(deprecated)]
    base64::encode(input)
}

pub fn b64_decode_old<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    #[allow(deprecated)]
    base64::decode(input)
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b64_encode() {
        let input = "hello world";
        let expected = "aGVsbG8gd29ybGQ=";
        let result = b64_encode(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_b64_encode_results() {
        assert_eq!(b64_encode(""), b64_encode_old(""));
        assert_eq!(b64_encode("hello world"), b64_encode_old("hello world"));
        assert_eq!(b64_encode("lorem ipsum"), b64_encode_old("lorem ipsum"));
    }

    #[test]
    fn test_b64_encode_byte_results() {
        assert_eq!(b64_encode([]), b64_encode_old([]));

        const BIN_ARRAY_ACCOUNT_DISCM: [u8; 8] = [92, 142, 92, 220, 5, 148, 70, 181];
        assert_eq!(
            b64_encode(BIN_ARRAY_ACCOUNT_DISCM),
            b64_encode_old(BIN_ARRAY_ACCOUNT_DISCM)
        );
    }

    #[test]
    fn test_b64_decode_byte_results() {
        assert_eq!(b64_decode(""), b64_decode_old(""));

        const LB_PAIR_BASE64: &str =  "IQsxYrVlsQ3oAywBsASIE0wdAADwSQIAp/v//1kEAAD0AQAAAAAAAAAAAAAAAAAAnP///wAAAAAIcLJnAAAAAAAAAAAAAAAA+5ABAJz///+QAQAB6AMAAJMmDTv5sOFnGOK8UDeWrHSlnumRczjq8f81v+EihAEmxvp6877brTo9ZfNqq8l0MbG75MLS9uDkfKYCA0UvXWF5Fvybt9mNK7bAVn5Zuvxzk/zWvZ7PfTAoc+KCLR4Y7AdkpBvrCZrZGyBajRza5liaTrE0KY793czXuyodXuXkkpAOPwAAAAAlix0DAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADJlulkTzStyLAycrxnlBV9f2K0qZUmQpeTqlz+T3vNoAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANVzamcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACUJ90TRaZRRmcqToc084e4l1qQLwbLB9LSTmFWNNwAbAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
        assert_eq!(b64_decode(LB_PAIR_BASE64), b64_decode_old(LB_PAIR_BASE64));
    }
}
