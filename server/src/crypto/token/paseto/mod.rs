pub mod v1;
pub mod v2;

pub mod error;
pub mod token;

pub use token::{
    Data as DataToken,
    SerializedData as SerializedDataToken,
    Packed as PackedToken,
    Header as TokenHeader,
    Unpacked as UnpackedToken,
};

enum Version {
    V1(v1::Type),
    V2(v2::Type),
}

pub enum ExpansionError {
}
pub struct Expanded {
    paseto_version: Version,
}
pub fn decrypt_phase_one<T>(array: &[u8]) -> Result<Expanded, ExpansionError> {
    Ok(Expanded {
        paseto_version: Version::V2(v2::Type::Public(v2::public::Token)),
    })
}

pub fn append_u64_to_little_endian_byte_array(to_encode: u64, byte_array: &mut [u8]) -> Result<(), &'static str> {
    const U64_BYTE_WIDTH: usize = 8;
    const BYTE_BIT_WIDTH: usize = 8;
    const U64_BIT_WIDTH: usize = U64_BYTE_WIDTH * BYTE_BIT_WIDTH;
    const U64_HIGH_BIT_MASK: u64 = 0x1u64 << (U64_BIT_WIDTH - 1);

    if byte_array.len() < U64_BYTE_WIDTH {
        Err("")?;
    }

    let to_encode = to_encode & !U64_HIGH_BIT_MASK;

    for offset_byte_shift in 0..U64_BYTE_WIDTH {
        let offset_bit_shift = offset_byte_shift * BYTE_BIT_WIDTH;
        let to_encode_offset = to_encode >> offset_bit_shift;
        // cast should truncate, but just in case
        const LOW_BYTE_MASK: u64 = 0xff;
        let low_byte = (to_encode_offset & LOW_BYTE_MASK) as u8;
        byte_array[offset_byte_shift] = low_byte;
    }
    Ok(())
}
pub fn multi_part_pre_auth_encoding(pieces: &[&[u8]]) -> Result<Vec<u8>, &'static str> {

    // precalc size
    const HEADER_SIZE: usize = 8;
    let mut total_size = 0;
    total_size += HEADER_SIZE + (HEADER_SIZE * pieces.len());
    for piece in pieces.iter() {
        total_size += piece.len();
    }

    // alloc and append
    let mut buffer = vec![0; total_size];
    let mut current_position = 0;

    let next_position = current_position + HEADER_SIZE;
    append_u64_to_little_endian_byte_array(pieces.len() as u64, &mut buffer[current_position..next_position])?;
    current_position = next_position;

    for piece in pieces.iter() {

        let next_position = current_position + HEADER_SIZE;
        append_u64_to_little_endian_byte_array(piece.len() as u64, &mut buffer[current_position..next_position])?;
        current_position = next_position;

        let next_position = current_position + piece.len();
        buffer[current_position..next_position].copy_from_slice(piece);
        current_position = next_position;

    }

    Ok(buffer)
}

pub fn collapse_to_vec(data: &[&[u8]]) -> Vec<u8> {
    data.iter().flat_map(|s| s.iter()).map(|b| *b).collect()
}

pub trait KnownClaims {}
impl KnownClaims for String {}

#[cfg(test)]
mod unit_test {
    use super::*;
    #[test]
    fn test_le64() {
        let mut buffer = [0u8; 8];
        append_u64_to_little_endian_byte_array(0, &mut buffer).unwrap();
        assert_eq!(
            vec![0, 0, 0, 0, 0, 0, 0, 0],
            buffer,
        );
        append_u64_to_little_endian_byte_array(!0u64, &mut buffer).unwrap();
        assert_eq!(
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0b01111111],
            buffer,
        );
    }
    #[cfg(test)]
    fn as_u8_vec(hex_str: &str) -> Result<Vec<u8>, ()> {
        let digits = (hex_str
            .chars()
            .map(|hex_char|
                 hex_char.to_digit(16)
                    .map(|i| i as u8)
                    .ok_or(())
            )
            .collect::<Result<Vec<_>, _>>())?;
        let starting_idx = digits.len() % 2;
        let mut bytes: Vec<u8> = (starting_idx..digits.len())
            .step_by(2)
            .map(|idx| {
                ((digits[idx] << 4) + digits[idx + 1]) as u8
            })
            .collect();
        if starting_idx == 1 {
            bytes.insert(0, digits[0]);
        }
        Ok(bytes)
    }
    #[test]
    fn test_pae() {
        let test_cases = vec![
            ("0000000000000000", vec![]),
            ("01000000000000000000000000000000", vec![""]),
            ("020000000000000000000000000000000000000000000000", vec!["", ""]),
            ("0100000000000000070000000000000050617261676f6e", vec!["Paragon"]),
            ("0200000000000000070000000000000050617261676f6e0a00000000000000496e6974696174697665", vec!["Paragon", "Initiative"]),
        ];
        // Constants taken from paseto source.
        for (solution, input) in test_cases {
            let data = &input
                .iter()
                .map(|string| string.as_bytes())
                .collect::<Vec<&[u8]>>()[..];
            assert_eq!(
                as_u8_vec(solution).unwrap(),
                multi_part_pre_auth_encoding(&data).unwrap(),
            );
        }
    }
}

