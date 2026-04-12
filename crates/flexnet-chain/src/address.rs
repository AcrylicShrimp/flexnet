#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address([u8; 32]);

impl Address {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Address;

    #[test]
    fn exposes_underlying_bytes() {
        let address = Address::new([3; 32]);

        assert_eq!(address.as_bytes(), &[3; 32]);
    }
}
