#![cfg_attr(not(test), no_std)]

use core::{mem::size_of, convert::TryInto};

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

fn inner_encode<const NUM_BYTES: usize>(mut value: u64, bytes: &mut [u8; 8]) -> usize {
    #[cfg(test)] eprintln!("A value: {value}");

    value -= OFFSETS[NUM_BYTES - 1];
    value <<= 1;
    value += 1;
    value <<= (NUM_BYTES - 1);
    #[cfg(test)] eprintln!("B value: {value}");
    *bytes = u64::to_le_bytes(value);
    #[cfg(test)] eprintln!("bytes: {bytes:?}");
    NUM_BYTES
}

/// Encodes a u64 into 1-9 bytes and returns the number of bytes updated.
pub fn encode(value: u64, buf: &mut [u8; 9]) -> usize {
    let low64: &mut [u8; size_of::<u64>()] = (&mut buf[..(size_of::<u64>())]).try_into().unwrap();

    // Waiting for answer to: https://stackoverflow.com/questions/75496635/how-to-get-a-workling-mutable-reference-to-a-subset-of-an-array
    match value {
        // FIXME: Change to exclusive ranges once the feature's stabilised.
        OFFSET0..=OFFSET1_LESS_ONE => inner_encode::<1>(value, low64),
        OFFSET1..=OFFSET2_LESS_ONE => inner_encode::<2>(value, low64),
        OFFSET2..=OFFSET3_LESS_ONE => inner_encode::<3>(value, low64),
        OFFSET3..=OFFSET4_LESS_ONE => inner_encode::<4>(value, low64),
        OFFSET4..=OFFSET5_LESS_ONE => inner_encode::<5>(value, low64),
        OFFSET5..=OFFSET6_LESS_ONE => inner_encode::<6>(value, low64),
        OFFSET6..=OFFSET7_LESS_ONE => inner_encode::<7>(value, low64),
        OFFSET7..=OFFSET8_LESS_ONE => inner_encode::<8>(value, low64),
        _ => { // All nine bytes are needed.
            *low64 = [0; size_of::<u64>()];
            drop(low64);

            let high64: &mut [u8; size_of::<u64>()] = (&mut buf[1..(size_of::<u64>()+1)]).try_into().unwrap();
            *high64 = u64::to_le_bytes(value);
            
            #[cfg(test)] eprintln!("9 value: {value}");
            #[cfg(test)] eprintln!("9 high64: {high64:?}");

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

        assert_eq!(OFFSET0, 0);
        assert_eq!(encode(0, &mut buf), 1);
        assert_eq!(buf, [0b00000001u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(1, &mut buf), 1);
        assert_eq!(buf, [0b00000011u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(127, &mut buf), 1);
        assert_eq!(buf, [0b11111111u8, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(OFFSET1, 128);
        assert_eq!(encode(128, &mut buf), 2);
        assert_eq!(buf, [0b00000010u8, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(129, &mut buf), 2);
        assert_eq!(buf, [0b00000110u8, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(16_511, &mut buf), 2);
        assert_eq!(buf, [0b11111110u8, 0xFF, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(OFFSET2, 16_512);
        assert_eq!(encode(16_512, &mut buf), 3);
        assert_eq!(buf, [0b00000100u8, 0x00, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(16_513, &mut buf), 3);
        assert_eq!(buf, [0b00001100u8, 0x00, 0x00, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(encode(OFFSET3 - 1, &mut buf), 3);
        assert_eq!(buf, [0b11111100u8, 0xFF, 0xFF, /* ignored */ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        assert_eq!(OFFSET3, 2_113_664);
        assert_eq!(OFFSET4, 270_549_120);
        assert_eq!(OFFSET5, 34_630_287_488);
        assert_eq!(OFFSET6, 4_432_676_798_592);
        assert_eq!(encode(OFFSET6, &mut buf), 7);
        assert_eq!(buf, [0b01000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00, 0x00]);
        assert_eq!(encode(OFFSET6 + 1, &mut buf), 7);
        assert_eq!(buf, [0b11000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00, 0x00]);
        assert_eq!(encode(OFFSET7 - 1, &mut buf), 7);
        assert_eq!(buf, [0b11000000u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* ignored */ 0x00, 0x00]);

        assert_eq!(OFFSET7, 567_382_630_219_904);
        assert_eq!(encode(OFFSET7, &mut buf), 8);
        assert_eq!(buf, [0b10000000u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00]);
        assert_eq!(encode(OFFSET7 + 1, &mut buf), 8);
        assert_eq!(buf, [0b10000000u8, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* ignored */ 0x00]);
        assert_eq!(encode(OFFSET8 - 1, &mut buf), 8);
        assert_eq!(buf, [0b10000000u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, /* ignored */ 0x00]);

        assert_eq!(OFFSET8, 72_624_976_668_147_840);
        assert_eq!(encode(OFFSET8, &mut buf), 9);
        assert_eq!(buf, [0b00000000u8, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01]);
        assert_eq!(encode(OFFSET8 + 1, &mut buf), 9);
        assert_eq!(buf, [0b00000000u8, 0x81, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01]);
        assert_eq!(encode(u64::MAX, &mut buf), 9);
        assert_eq!(buf, [0b00000000u8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }
}
