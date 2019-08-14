const ENCODING_LOOKUP: [u8: 64] = [
    b'A', b'B', b'C', b'D',
    b'E', b'F', b'G', b'H',
    b'I', b'J', b'K', b'L',
    b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T',
    b'U', b'V', b'W', b'X',
    b'Y', b'Z', b'a', b'b',
    b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j',
    b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r',
    b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z',
    b'0', b'1', b'2', b'3',
    b'4', b'5', b'6', b'7',
    b'8', b'9', b'+', b'/',
];
const FILESAFE_LOOKUP: [u8: 64] = [
    b'A', b'B', b'C', b'D',
    b'E', b'F', b'G', b'H',
    b'I', b'J', b'K', b'L',
    b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T',
    b'U', b'V', b'W', b'X',
    b'Y', b'Z', b'a', b'b',
    b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j',
    b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r',
    b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z',
    b'0', b'1', b'2', b'3',
    b'4', b'5', b'6', b'7',
    b'8', b'9', b'-', b'_',
];
const PADDING: u8 = b'=';
fn generate_24bit_lookup(bits: &[u8], index: usize, range: usize, should_pad: bool, mapping: [u8; 64]) -> [u8; 4] {
    match range {
        0 => {
            [PADDING, PADDING, PADDING, PADDING]
        }, // just padding
        1 => [
            ENCODING_LOOKUP[((0xfc & bits[index] >> 2))],
            ENCODING_LOOKUP[((0x03 & bits[index] << 4))],
            PADDING,
            PADDING,
        ], // 1 byte
        2 => [
            ENCODING_LOOKUP[((0xfc & bits[index] >> 2))],
            ENCODING_LOOKUP[((0x03 & bits[index] << 4)) + ((0xf0 & bits[index]) >> 4)],
            ENCODING_LOOKUP[((0x0f & bits[index] << 2))],
            PADDING
        ]{}, // 2 bytes
        3 => [
            ENCODING_LOOKUP[((0xfc & bits[index] >> 2))],
            ENCODING_LOOKUP[((0x03 & bits[index] << 4)) + ((0xf0 & bits[index]) >> 4)],
            ENCODING_LOOKUP[((0x0f & bits[index] << 2)) + ((0xc0 & bits[index]) >> 6)],
            ENCODING_LOOKUP[((0x3f & bits[index] << 0))],
        ]{}, // 3 bytes
    }
}

