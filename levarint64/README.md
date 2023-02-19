# LeVarInt64

LeVarInt64 is a library for encoding and decoding u64s (and i64s) in usually fewer than eight bytes.

Typically this is used to serialise a "count" field, when the number of following bytes is unknown at compile-time.
The count will most often take just one or two bytes to serialise, but u64::MAX will take nine bytes.

The encoded format is designed for efficient encoding and decoding on little-endian architectures:

```
                   Encoding                                 Values
    
b[8] b[7] b[6] b[5] b[4] b[3] b[2] b[1]    b[0]  
    
                                        0b???????1           0 - 127
                                   0x?? 0b??????10         128 - 16,511
                              0x?? 0x?? 0b?????100      16,512 - 2,113,663
                         0x?? 0x?? 0x?? 0b????1000   2,113,663 - 270,549,119
                    0x?? 0x?? 0x?? 0x?? 0b???10000 270,549,120 - 3.5e10
               0x?? 0x?? 0x?? 0x?? 0x?? 0b??100000      3.5e10 - 4.4e12
          0x?? 0x?? 0x?? 0x?? 0x?? 0x?? 0b?1000000      4.4e12 - 5.7e14
     0x?? 0x?? 0x?? 0x?? 0x?? 0x?? 0x?? 0b10000000      5.7e14 - 7.3e16
0x?? 0x?? 0x?? 0x?? 0x?? 0x?? 0x?? 0x?? 0b00000000           0 - 1.8e19
     \___________________low_u64_________________/
\______________high_u64_______________/
```

Trailing zeros as the length indicator has been chosen because:
- Counting trailing zeros is the most efficient bit count on the x86_64 architecture.
- It is the second most efficient bit count on Aarch64, needing just one additional instruction.
- [Assembly instructions for counting bits.](https://stackoverflow.com/a/75335655/129805)

Decoding:
- If low_u64.trailing_zeros() > 7, return high_u64.
- Else:
  - Shift-left to wipe unneeded top bits.
  - Shift-right to wipe the final one and repeated zeros.
  - Add the offset, based on the earlier number of trailing zeros.

Encoding:
- If value > 5.7e14, write 0x00 and the eight-byte value (high_u64).
- Else:
  - Calculate how many bytes will be required to store it.
  - Subtract the offset from the value.
  - Left-shift one place and set the lowest bit.
  - Left-shift the remaining places, which will be filled with zeros.
  - Write the value as 1-8 bytes (low_u64).

Notes:
- LeVarInts are inspired by Protobuf's VarInts, but use a little-endian format to allow more efficient processing.
- Unlike VarInts, LeVarInts use offsets to gain ~1% space efficiency at the cost of one assembly "add/sub  immediate" instruction.
- The nine-byte encoding is a special case.
  - If the pattern has been followed:
    - it would use 73 bits (10 bytes) instead of nine, and
    - a zero value in the high 64 bits would represent OFFSET8 instead being a duplicate zero.
  - Instead, as we know that a u64 cannot exceed 8 bytes:
    - we omit the shifts, and
    - just store the eight-byte value verbatim in the high eight bytes.
  - A LeVarInt128 would be more complex.

API:
- For the first release of the crate, only the very low-level encode_u64_to_array_ref() and decode_u64_from_array_ref() are provided.
  - These are most efficient, as bounds checks are optimised away.
  - They are unfriendly.
  - What is a better API?
    - [u8] slices?
    - Iterators?

Thanks:
- To Masklinn and Chayim Friedman for helping me with https://stackoverflow.com/questions/75370230
- To PitaJ for answering https://stackoverflow.com/questions/75496635

