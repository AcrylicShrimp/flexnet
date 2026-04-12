use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainId(u16);

impl ChainId {
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    pub const fn into_u16(self) -> u16 {
        self.0
    }
}

impl Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04x}", self.0)
    }
}
