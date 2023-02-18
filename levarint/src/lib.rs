#![cfg_attr(not(test), no_std)]

use core::mem::size_of;

/// Calculate the offsets.
const OFFSET0: u64 = 0;
const OFFSET1: u64 = OFFSET0 + (1 << 7);
const OFFSET2: u64 = OFFSET1 + (1 << 14);
const OFFSET3: u64 = OFFSET2 + (1 << 21);
const OFFSET4: u64 = OFFSET3 + (1 << 28);
const OFFSET5: u64 = OFFSET4 + (1 << 35);
const OFFSET6: u64 = OFFSET5 + (1 << 42);
const OFFSET7: u64 = OFFSET6 + (1 << 49);
const OFFSET8: u64 = OFFSET7 + (1 << 56);
const OFFSETS: [u64; 8] = [OFFSET0, OFFSET1, OFFSET2, OFFSET3, OFFSET4, OFFSET5, OFFSET6, OFFSET7];

/// Calculate one less than the offsets, as matching on exclusive ranges is not yet a stable feature.
const OFFSET1_LESS_ONE: u64 = OFFSET1 - 1;
const OFFSET2_LESS_ONE: u64 = OFFSET2 - 1;
const OFFSET3_LESS_ONE: u64 = OFFSET3 - 1;
const OFFSET4_LESS_ONE: u64 = OFFSET4 - 1;
const OFFSET5_LESS_ONE: u64 = OFFSET5 - 1;
const OFFSET6_LESS_ONE: u64 = OFFSET6 - 1;
const OFFSET7_LESS_ONE: u64 = OFFSET7 - 1;
const OFFSET8_LESS_ONE: u64 = OFFSET8 - 1;

/// The left and right shift distances, for different sizes of encodings.
const LEFTS: [u32; 8] = [56, 48, 40, 32, 24, 16, 8, 0];
const RIGHTS: [u32; 8] = [57, 50, 43, 36, 29, 22, 15, 8];

fn inner_decode<const NUM_BYTES: u32>(mut value: u64) -> u64 {
    let left_shift = LEFTS[(NUM_BYTES - 1) as usize];
    //eprintln!("left_shift: {left_shift}");
    value <<= left_shift;

    let right_shift = RIGHTS[(NUM_BYTES - 1) as usize];
    //eprintln!("right_shift: {right_shift}");
    value >>= right_shift;

    value + OFFSETS[(NUM_BYTES - 1) as usize]
}

fn u64_from_low_eight(buf: &[u8; 9]) -> u64 {
    let bytes: &[u8; size_of::<u64>()] = buf[..size_of::<u64>()].try_into().unwrap();
    u64::from_le_bytes(*bytes)
}

fn u64_from_high_eight(buf: &[u8; 9]) -> u64 {
    let bytes: &[u8; size_of::<u64>()] = buf[1..(size_of::<u64>()+1)].try_into().unwrap();
    u64::from_le_bytes(*bytes)
}

/// Decodes 1-9 bytes and returns a u64 and the number of bytes consumed.
pub fn decode(buf: &[u8; 9]) -> (u64, usize) {
    let low64 = u64_from_low_eight(buf);
    //eprintln!("low64: {low64}");
    let trailing_zeros = low64.trailing_zeros();

    match trailing_zeros {
        0 => (inner_decode::<1>(low64), 1),
        1 => (inner_decode::<2>(low64), 2),
        2 => (inner_decode::<3>(low64), 3),
        3 => (inner_decode::<4>(low64), 4),
        4 => (inner_decode::<5>(low64), 5),
        5 => (inner_decode::<6>(low64), 6),
        6 => (inner_decode::<7>(low64), 7),
        7 => (inner_decode::<8>(low64), 8),
        _ => {
            let high64 = u64_from_high_eight(buf);
            //eprintln!("high64: {high64}");
            (high64, 9) // All nine bytes were used.
        }
    }
}

fn u64_to_slice<const NUM_BYTES: usize>(value: u64, slice: &mut [u8]) {
    // TODO: We only need to write NUM_BYTES, not all of them.
    unsafe {
        *slice.get_unchecked_mut(0) = value as u8;
        *slice.get_unchecked_mut(1) = (value << 8) as u8;
        *slice.get_unchecked_mut(2) = (value << 16) as u8;
        *slice.get_unchecked_mut(3) = (value << 24) as u8;
        *slice.get_unchecked_mut(4) = (value << 32) as u8;
        *slice.get_unchecked_mut(5) = (value << 40) as u8;
        *slice.get_unchecked_mut(6) = (value << 48) as u8;
        *slice.get_unchecked_mut(7) = (value << 56) as u8;
    }
}

fn inner_encode<const NUM_BYTES: usize>(mut value: u64, slice: &mut [u8]) -> usize {
    value -= OFFSETS[NUM_BYTES - 1];
    value <<= 1;
    value += 1;
    value <<= NUM_BYTES - 1;
    u64_to_slice::<NUM_BYTES>(value, slice);
    NUM_BYTES
}

/// Encodes a u64 into 1-9 bytes and returns the number of bytes updated.
pub fn encode(value: u64, buf: &mut [u8; 9]) -> usize {
    match value {
        // FIXME: Change to exclusive ranges once the feature's stabilised.
        OFFSET0..=OFFSET1_LESS_ONE => inner_encode::<1>(value, &mut buf[0..1]),
        OFFSET1..=OFFSET2_LESS_ONE => inner_encode::<2>(value, &mut buf[0..2]),
        OFFSET2..=OFFSET3_LESS_ONE => inner_encode::<3>(value, &mut buf[0..3]),
        OFFSET3..=OFFSET4_LESS_ONE => inner_encode::<4>(value, &mut buf[0..4]),
        OFFSET4..=OFFSET5_LESS_ONE => inner_encode::<4>(value, &mut buf[0..5]),
        OFFSET5..=OFFSET6_LESS_ONE => inner_encode::<5>(value, &mut buf[0..6]),
        OFFSET6..=OFFSET7_LESS_ONE => inner_encode::<6>(value, &mut buf[0..7]),
        OFFSET7..=OFFSET8_LESS_ONE => inner_encode::<8>(value, &mut buf[0..8]),
        _ => { // All nine bytes are needed.
            buf[0] = 0;
            buf[1] = value as u8;
            buf[2] = (value << 8) as u8;
            buf[3] = (value << 16) as u8;
            buf[4] = (value << 24) as u8;
            buf[5] = (value << 32) as u8;
            buf[6] = (value << 40) as u8;
            buf[7] = (value << 48) as u8;
            buf[8] = (value << 56) as u8;
            9
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_from_x() {
        assert_eq!(u64_from_low_eight(&[0,0,0,0,0,0,0,0,0]), 0);
        assert_eq!(u64_from_low_eight(&[1,0,0,0,0,0,0,0,0]), 1);
        assert_eq!(u64_from_low_eight(&[0,1,0,0,0,0,0,0,0]), 256);
        assert_eq!(u64_from_high_eight(&[0,1,0,0,0,0,0,0,0]), 1);
    }

    #[test]
    fn test_decoding() {
        assert_eq!(OFFSET0, 0);
        assert_eq!(decode(&[
            0b00000001u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (0, 1)); // OFFSET0
        assert_eq!(decode(&[
            0b00000011u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (1, 1)); // OFFSET0 + 1
        assert_eq!(decode(&[
            0b11111111u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (127, 1)); // OFFSET1 - 1

        assert_eq!(OFFSET1, 128);
        assert_eq!(decode(&[
            0b00000010u8, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (128, 2)); // OFFSET1
        assert_eq!(decode(&[
            0b00000110u8, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (129, 2)); // OFFSET1 + 1
        assert_eq!(decode(&[
            0b11111110u8, 0xFF, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (16_511, 2)); // OFFSET2 - 1

        assert_eq!(OFFSET2, 16_512);
        assert_eq!(decode(&[
            0b00000100u8, 0x00, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (16_512, 3)); // OFFSET2
        assert_eq!(decode(&[
            0b00001100u8, 0x00, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (16_513, 3)); // OFFSET2 + 1
        assert_eq!(decode(&[
            0b11111100u8, 0xFF, 0xFF, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (OFFSET3 - 1, 3));

        assert_eq!(OFFSET3, 2_113_664);
        assert_eq!(OFFSET4, 270_549_120);
        assert_eq!(OFFSET5, 34_630_287_488);
        assert_eq!(OFFSET6, 4_432_676_798_592);
        assert_eq!(decode(&[
            0b01000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00, 0x00
        ]), (OFFSET6, 7));
        assert_eq!(decode(&[
            0b11000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00, 0x00
        ]), (OFFSET6 + 1, 7));
        assert_eq!(decode(&[
            0b11000000u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* ignored */ 0x00, 0x00
        ]), (OFFSET7 - 1, 7));

        assert_eq!(OFFSET7, 567_382_630_219_904);
        assert_eq!(decode(&[
            0b10000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00
        ]), (OFFSET7, 8));
        assert_eq!(decode(&[
            0b10000000u8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00
        ]), (OFFSET7 + 1, 8));
        assert_eq!(decode(&[
            0b10000000u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* ignored */ 0x00
        ]), (OFFSET8 - 1, 8));

        assert_eq!(OFFSET8, 72_624_976_668_147_840);
        assert_eq!(decode(&[
            0b00000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (0, 9)); // supernormal encoding of zero in nine bytes
        assert_eq!(decode(&[
            0b00000000u8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
        ]), (1, 9)); // supernormal encoding of one in nine bytes
        assert_eq!(decode(&[
            0b00000000u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
        ]), (u64::MAX, 9));
    }

    #[test]
    fn test_encoding() {
        let mut buf = [0u8; 9];

        assert_eq!(encode(0, &mut buf), 1);
        assert_eq!(buf, [0b00000001u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }
}
