use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainVersion(u16);

impl ChainVersion {
    pub const fn new(version: u16) -> Self {
        Self(version)
    }

    pub const fn into_u16(self) -> u16 {
        self.0
    }
}

impl Display for ChainVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ChainVersion;

    #[test]
    fn displays_as_fixed_width_hex() {
        assert_eq!(ChainVersion::new(0x0011).to_string(), "0x0011");
    }
}
