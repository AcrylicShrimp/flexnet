use std::fmt::Display;

const MAGIC: &str = "FLEX";
const MAGIC_BYTES: [u8; 4] = *b"FLEX";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Magic;

impl Magic {
    pub const BYTES: usize = MAGIC.len();

    pub const fn as_str(self) -> &'static str {
        MAGIC
    }

    pub const fn as_bytes(self) -> [u8; Self::BYTES] {
        MAGIC_BYTES
    }
}

impl TryFrom<&[u8]> for Magic {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value == MAGIC_BYTES {
            Ok(Magic)
        } else {
            Err("invalid magic")
        }
    }
}

impl Display for Magic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", MAGIC)
    }
}
