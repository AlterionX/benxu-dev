//! Re-export version of a base64 encoding used throughout the library.

pub(crate) mod base64 {
    pub use base64::DecodeError;

    fn div_and_round_up(dividend: usize, divisor: usize) -> usize {
        (dividend + (divisor - 1)) / divisor
    }

    pub fn encode_no_padding(data: &[u8]) -> Vec<u8> {
        let mut encoded = vec![0; div_and_round_up(data.len(), 3) * 4];
        let bytes_written = base64::encode_config_slice(data, base64::STANDARD_NO_PAD, &mut encoded);
        encoded.resize(bytes_written, 0);
        encoded
    }
    pub fn decode_no_padding(data: &[u8]) -> Result<Vec<u8>, base64::DecodeError> {
        let mut decoded = vec![0; div_and_round_up(data.len(), 4) * 3];
        let bytes_written = base64::decode_config_slice(data, base64::STANDARD_NO_PAD, &mut decoded)?;
        decoded.resize(bytes_written, 0);
        Ok(decoded)
    }
}
