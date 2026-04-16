use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Timeout {
    pub height: u128,
    pub round: u32,
    pub instant: Instant,
}

pub async fn conditional_timeout(next_timeout: Option<&Timeout>) -> Option<&Timeout> {
    match next_timeout {
        Some(timeout) => {
            tokio::time::sleep_until(timeout.instant).await;
            Some(timeout)
        }
        None => None,
    }
}
