macro_rules! impl_to_le_bytes_primitive {
    ($ty:ty) => {
        impl crate::canonical_encode::CanonicalEncode for $ty {
            fn encode_to(
                &self,
                encoder: &mut crate::encoder::Encoder,
            ) -> Result<(), crate::encoder::EncoderError> {
                encoder.write_fixed_bytes(&self.to_le_bytes());
                Ok(())
            }
        }

        impl crate::canonical_decode::CanonicalDecode for $ty {
            fn decode_from(
                decoder: &mut crate::decoder::Decoder,
            ) -> Result<Self, crate::decoder::DecoderError> {
                Ok(<$ty>::from_le_bytes(*decoder.read_fixed_bytes()?))
            }
        }
    };
}

impl crate::canonical_encode::CanonicalEncode for bool {
    fn encode_to(
        &self,
        encoder: &mut crate::encoder::Encoder,
    ) -> Result<(), crate::encoder::EncoderError> {
        encoder.write_fixed_bytes(&[if *self { 1 } else { 0 }]);
        Ok(())
    }
}

impl crate::canonical_decode::CanonicalDecode for bool {
    fn decode_from(
        decoder: &mut crate::decoder::Decoder,
    ) -> Result<Self, crate::decoder::DecoderError> {
        let discriminant = decoder.read_u8()?;

        match discriminant {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(crate::decoder::DecoderError::InvalidDiscriminant {
                type_name: "bool",
                discriminant: u64::from(other),
            }),
        }
    }
}

impl_to_le_bytes_primitive!(i8);
impl_to_le_bytes_primitive!(i16);
impl_to_le_bytes_primitive!(i32);
impl_to_le_bytes_primitive!(i64);
impl_to_le_bytes_primitive!(i128);

impl_to_le_bytes_primitive!(u8);
impl_to_le_bytes_primitive!(u16);
impl_to_le_bytes_primitive!(u32);
impl_to_le_bytes_primitive!(u64);
impl_to_le_bytes_primitive!(u128);
