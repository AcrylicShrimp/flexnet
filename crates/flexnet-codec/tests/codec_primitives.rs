use flexnet_codec::{
    canonical_decode::CanonicalDecode,
    canonical_encode::CanonicalEncode,
    decoder::{Decoder, DecoderError},
    encoder::{Encoder, EncoderError},
};

#[test]
fn encoder_writes_integer_primitives_in_little_endian() {
    let mut encoder = Encoder::new();

    encoder.write_u8(0x12);
    encoder.write_u16_le(0x3456);
    encoder.write_u32_le(0x789a_bcde);
    encoder.write_u64_le(0x0123_4567_89ab_cdef);
    encoder.write_u128_le(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210);

    assert_eq!(
        encoder.finish(),
        vec![
            0x12, 0x56, 0x34, 0xde, 0xbc, 0x9a, 0x78, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23,
            0x01, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0xef, 0xcd, 0xab, 0x89, 0x67,
            0x45, 0x23, 0x01,
        ]
    );
}

#[test]
fn decoder_reads_integer_primitives_in_little_endian() {
    let input = [
        0x12, 0x56, 0x34, 0xde, 0xbc, 0x9a, 0x78, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01,
        0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23,
        0x01,
    ];
    let mut decoder = Decoder::new(&input);

    assert_eq!(decoder.read_u8(), Ok(0x12));
    assert_eq!(decoder.read_u16_le(), Ok(0x3456));
    assert_eq!(decoder.read_u32_le(), Ok(0x789a_bcde));
    assert_eq!(decoder.read_u64_le(), Ok(0x0123_4567_89ab_cdef));
    assert_eq!(
        decoder.read_u128_le(),
        Ok(0x0123_4567_89ab_cdef_fedc_ba98_7654_3210)
    );
    assert_eq!(decoder.finish(), Ok(()));
}

#[test]
fn fixed_bytes_round_trip_without_length_prefix() {
    let bytes = [0x10, 0x20, 0x30, 0x40];
    let mut encoder = Encoder::new();
    encoder.write_fixed_bytes(&bytes);

    let encoded = encoder.finish();
    assert_eq!(encoded, bytes);

    let mut decoder = Decoder::new(&encoded);
    assert_eq!(decoder.read_fixed_bytes::<4>(), Ok(&bytes));
    assert_eq!(decoder.finish(), Ok(()));
}

#[test]
fn dynamic_bytes_round_trip_with_u16_length_prefix() {
    let bytes = [0xaa, 0xbb, 0xcc];
    let mut encoder = Encoder::new();
    assert_eq!(encoder.write_dynamic_bytes(&bytes), Ok(()));

    let encoded = encoder.finish();
    assert_eq!(encoded, [0x03, 0x00, 0xaa, 0xbb, 0xcc]);

    let mut decoder = Decoder::new(&encoded);
    assert_eq!(decoder.read_dynamic_bytes(), Ok(bytes.as_slice()));
    assert_eq!(decoder.finish(), Ok(()));
}

#[test]
fn dynamic_bytes_accepts_maximum_u16_length() {
    let bytes = vec![0x5a; u16::MAX as usize];
    let mut encoder = Encoder::new();

    assert_eq!(encoder.write_dynamic_bytes(&bytes), Ok(()));

    let encoded = encoder.finish();
    assert_eq!(&encoded[..2], &u16::MAX.to_le_bytes());

    let mut decoder = Decoder::new(&encoded);
    assert_eq!(decoder.read_dynamic_bytes(), Ok(bytes.as_slice()));
    assert_eq!(decoder.finish(), Ok(()));
}

#[test]
fn dynamic_bytes_rejects_length_above_u16_range() {
    let bytes = vec![0; u16::MAX as usize + 1];
    let mut encoder = Encoder::new();

    assert_eq!(
        encoder.write_dynamic_bytes(&bytes),
        Err(EncoderError::DynamicBytesLengthExceedsCanonicalU16Range)
    );
    assert!(encoder.finish().is_empty());
}

#[test]
fn decoder_rejects_truncated_fixed_width_values() {
    let mut u8_decoder = Decoder::new(&[]);
    assert!(matches!(
        u8_decoder.read_u8(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));

    let mut u16_decoder = Decoder::new(&[0; 1]);
    assert!(matches!(
        u16_decoder.read_u16_le(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));

    let mut u32_decoder = Decoder::new(&[0; 3]);
    assert!(matches!(
        u32_decoder.read_u32_le(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));

    let mut u64_decoder = Decoder::new(&[0; 7]);
    assert!(matches!(
        u64_decoder.read_u64_le(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));

    let mut u128_decoder = Decoder::new(&[0; 15]);
    assert!(matches!(
        u128_decoder.read_u128_le(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));
}

#[test]
fn failed_fixed_width_read_does_not_advance_decoder() {
    let mut decoder = Decoder::new(&[0xab]);

    assert!(matches!(
        decoder.read_u16_le(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));
    assert_eq!(decoder.read_u8(), Ok(0xab));
    assert_eq!(decoder.finish(), Ok(()));
}

#[test]
fn decoder_rejects_truncated_dynamic_bytes() {
    let mut truncated_length = Decoder::new(&[0x03]);
    assert!(matches!(
        truncated_length.read_dynamic_bytes(),
        Err(DecoderError::InputExhausted { bytes_consumed: 0 })
    ));

    let mut truncated_payload = Decoder::new(&[0x03, 0x00, 0xaa, 0xbb]);
    assert!(matches!(
        truncated_payload.read_dynamic_bytes(),
        Err(DecoderError::InputExhausted { bytes_consumed: 2 })
    ));
}

#[derive(Debug, PartialEq, Eq)]
struct OneByte(u8);

impl CanonicalEncode for OneByte {
    fn encode_to(&self, encoder: &mut Encoder) -> Result<(), EncoderError> {
        encoder.write_u8(self.0);
        Ok(())
    }
}

impl CanonicalDecode for OneByte {
    fn decode_from(decoder: &mut Decoder) -> Result<Self, DecoderError> {
        Ok(Self(decoder.read_u8()?))
    }
}

#[test]
fn canonical_traits_round_trip_a_value() {
    let value = OneByte(0x42);

    let encoded = value.encode_canonical().unwrap();
    assert_eq!(encoded, [0x42]);
    assert_eq!(OneByte::decode_canonical(&encoded), Ok(value));
}

#[test]
fn canonical_decode_rejects_trailing_bytes() {
    assert_eq!(
        OneByte::decode_canonical(&[0x42, 0xff]),
        Err(DecoderError::InputNotFullyConsumed {
            bytes_consumed: 1,
            bytes_remaining: 1,
        })
    );
}
