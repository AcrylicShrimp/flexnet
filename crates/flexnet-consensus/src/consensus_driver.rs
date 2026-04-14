use std::sync::mpsc::Receiver;

use crate::{
    consensus_config::ConsensusConfig,
    ports::{block_port::BlockPort, message_port::MessagePort},
    proposal::Proposal,
    proposal_validator::ProposalValidator,
    state_machine::StateMachine,
};

pub struct ConsensusDriver<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    config: ConsensusConfig,
    state_machine: StateMachine<P, V>,
}

impl<P, V> ConsensusDriver<P, V>
where
    P: Proposal,
    V: ProposalValidator<P>,
{
    pub fn new(config: ConsensusConfig, proposal_validator: V) -> Self {
        Self {
            config: config.clone(),
            state_machine: StateMachine::new(0, config, proposal_validator).unwrap(),
        }
    }

    pub fn run(&mut self) {}

    pub fn stop(self) {}
}

fn driver_loop(
    message_port: impl MessagePort,
    block_port: impl BlockPort,
    stop_signal: Receiver<()>,
) {
    loop {}
}
