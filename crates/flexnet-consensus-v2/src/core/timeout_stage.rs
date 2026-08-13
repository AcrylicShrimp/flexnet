#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreTimeoutStage {
    Propose,
    Prevote,
    Precommit,
}
