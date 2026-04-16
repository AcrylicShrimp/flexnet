mod make_messages;
mod message_to_state_input;
mod proposal_block;
mod proposal_block_validator;
mod proposal_generator;
mod run_state_machine;
mod timeout;

use crate::{
    consensus_config::ConsensusConfig,
    consensus_driver::{
        message_to_state_input::message_to_state_input,
        proposal_block::ProposalBlock,
        proposal_block_validator::ProposalBlockValidator,
        proposal_generator::ProposalGenerator,
        run_state_machine::{StateMachineExecutionContext, run_state_machine},
        timeout::{Timeout, conditional_timeout},
    },
    message::Message,
    ports::{block_port::BlockPort, chain_port::ChainPort, message_port::MessagePort},
    state_input::StateInput,
    state_machine::{StateMachine, StateMachineInitError},
};
use flexnet_chain::chain_config::ChainConfig;
use std::sync::Arc;
use thiserror::Error;
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};

struct JobContext {
    stop_signal_sender: Sender<()>,
    join_handle: JoinHandle<()>,
}

pub struct ConsensusDriver {
    chain_config: Arc<ChainConfig>,
    consensus_config: Arc<ConsensusConfig>,
    job_context: Option<JobContext>,
}

#[derive(Error, Debug)]
pub enum ConsensusDriverStartError {
    #[error("already running")]
    AlreadyRunning,
    #[error("state machine initialization failed: {0}")]
    StateMachineInitError(#[from] StateMachineInitError),
}

impl ConsensusDriver {
    pub fn new(chain_config: ChainConfig, consensus_config: ConsensusConfig) -> Self {
        let chain_config = Arc::new(chain_config);
        let consensus_config = Arc::new(consensus_config);

        Self {
            chain_config,
            consensus_config,
            job_context: None,
        }
    }

    pub fn run(
        &mut self,
        height: u128,
        message_port: impl MessagePort,
        block_port: impl BlockPort,
        chain_port: impl ChainPort,
    ) -> Result<(), ConsensusDriverStartError> {
        if self.job_context.is_some() {
            return Err(ConsensusDriverStartError::AlreadyRunning);
        }

        let (stop_signal_sender, stop_signal_receiver) = tokio::sync::mpsc::channel(1);
        let join_handle = tokio::spawn(driver_loop(
            height,
            StateMachine::new(
                self.chain_config.clone(),
                self.consensus_config.clone(),
                ProposalBlockValidator,
            )?,
            self.chain_config.clone(),
            self.consensus_config.clone(),
            message_port,
            block_port,
            chain_port,
            stop_signal_receiver,
        ));

        self.job_context = Some(JobContext {
            stop_signal_sender,
            join_handle,
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        let job_context = match self.job_context.take() {
            Some(job_context) => job_context,
            None => return,
        };

        let _ = job_context.stop_signal_sender.send(()).await;
        let _ = job_context.join_handle.await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn driver_loop(
    initial_height: u128,
    mut state_machine: StateMachine<ProposalBlock, ProposalBlockValidator>,
    chain_config: Arc<ChainConfig>,
    consensus_config: Arc<ConsensusConfig>,
    mut message_port: impl MessagePort,
    block_port: impl BlockPort,
    chain_port: impl ChainPort,
    mut stop_signal: Receiver<()>,
) {
    let mut next_timeout: Option<Timeout> = None;
    let (proposal_generator, mut proposal_receiver) =
        ProposalGenerator::new(block_port, consensus_config.clone());

    run_state_machine(
        StateInput::StartHeight {
            height: initial_height,
        },
        StateMachineExecutionContext {
            chain_config: &chain_config,
            consensus_config: &consensus_config,
            timeout: &mut next_timeout,
            message_port: &mut message_port,
            proposal_generator: &proposal_generator,
            chain_port: &chain_port,
        },
        &mut state_machine,
    );

    loop {
        let state_input = select! {
            Some(proposal) = proposal_receiver.recv() => {
                message_to_state_input(Message::Propose(proposal), &state_machine, &chain_config, &consensus_config).ok()
            }
            Some(message) = message_port.receiver().recv() => {
                message_to_state_input(message, &state_machine, &chain_config, &consensus_config).ok()
            }
            Some(timeout) = conditional_timeout(next_timeout.as_ref()) => {
                let state_input = StateInput::RoundTimeout {
                    height: timeout.height,
                    round: timeout.round,
                };
                next_timeout = None;
                Some(state_input)
            }
            _ = stop_signal.recv() => {
                break;
            }
        };

        let state_input = match state_input {
            Some(state_input) => state_input,
            None => {
                continue;
            }
        };

        run_state_machine(
            state_input,
            StateMachineExecutionContext {
                chain_config: &chain_config,
                consensus_config: &consensus_config,
                timeout: &mut next_timeout,
                message_port: &mut message_port,
                proposal_generator: &proposal_generator,
                chain_port: &chain_port,
            },
            &mut state_machine,
        );
    }
}
