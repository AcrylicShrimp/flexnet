use std::fmt::Display;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Domain {
    Message = 0x10,
    Block = 0x20,
    Tx = 0x30,
}

impl Domain {
    pub const BYTES: usize = 1;

    pub const fn into_u8(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Domain {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Domain::Message),
            0x20 => Ok(Domain::Block),
            0x30 => Ok(Domain::Tx),
            _ => Err("invalid domain"),
        }
    }
}

impl Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Domain::Message => "msg",
                Domain::Block => "blk",
                Domain::Tx => "tx ",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Domain;

    #[test]
    fn displays_as_fixed_width() {
        assert_eq!(Domain::Message.to_string(), "msg");
        assert_eq!(Domain::Block.to_string(), "blk");
        assert_eq!(Domain::Tx.to_string(), "tx ");
    }
}
