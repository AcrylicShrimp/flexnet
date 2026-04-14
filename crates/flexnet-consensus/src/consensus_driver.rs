mod proposal_block;
mod proposal_block_validator;

use crate::{
    consensus_config::ConsensusConfig,
    consensus_driver::{
        proposal_block::ProposalBlock, proposal_block_validator::ProposalBlockValidator,
    },
    justification::{Evidence, Justification},
    message::{Message, MessageVerificationError},
    messages::{
        msg_precommit::{MsgPrecommit, PrecommitPayload},
        msg_prevote::{MsgPrevote, PrevotePayload},
        msg_propose::{
            MsgPropose, ProposeEvidencePayload, ProposeJustificationPayload, ProposePayload,
        },
    },
    polka::Polka,
    ports::{block_port::BlockPort, chain_port::ChainPort, message_port::MessagePort},
    state_input::StateInput,
    state_machine::{StateMachine, StateMachineInitError},
    state_output::StateOutput,
};
use flexnet_chain::{chain_config::ChainConfig, crypto::sign, hash::Hash};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
    select,
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
    time::Instant,
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
            StateMachine::new(
                height,
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

async fn driver_loop(
    mut state_machine: StateMachine<ProposalBlock, ProposalBlockValidator>,
    chain_config: Arc<ChainConfig>,
    consensus_config: Arc<ConsensusConfig>,
    message_port: impl MessagePort,
    block_port: impl BlockPort,
    chain_port: impl ChainPort,
    mut stop_signal: Receiver<()>,
) {
    struct Timeout {
        height: u128,
        round: u32,
        instant: Instant,
    }

    let message_sender = message_port.sender();
    let mut message_receiver = message_port.receiver();
    let mut next_timeout: Option<Timeout> = None;

    async fn conditional_timeout(next_timeout: Option<&Timeout>) -> Option<&Timeout> {
        match next_timeout {
            Some(timeout) => {
                tokio::time::sleep_until(timeout.instant).await;
                Some(timeout)
            }
            None => None,
        }
    }

    loop {
        let state_input = select! {
            Some(message) = message_receiver.recv() => {
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

        let mut current_state_outputs = state_machine.step(state_input);

        while !current_state_outputs.is_empty() {
            let mut next_state_outputs = vec![];

            for state_output in current_state_outputs {
                match state_output {
                    StateOutput::StartTimeout {
                        height,
                        round,
                        timeout_ms,
                    } => {
                        next_timeout = Some(Timeout {
                            height,
                            round,
                            instant: Instant::now() + Duration::from_millis(timeout_ms),
                        });
                    }
                    StateOutput::StartRound { height, round } => {
                        next_state_outputs
                            .extend(state_machine.step(StateInput::StartRound { height, round }));
                    }
                    StateOutput::Propose {
                        height,
                        round,
                        polka,
                    } => {
                        let propose_message = make_propose_message(
                            height,
                            round,
                            polka,
                            &block_port,
                            &consensus_config,
                        );
                        let next_state_input = message_to_state_input(
                            Message::Propose(propose_message.clone()),
                            &state_machine,
                            &chain_config,
                            &consensus_config,
                        )
                        .expect("failed to convert local message to state input");

                        next_state_outputs.extend(state_machine.step(next_state_input));

                        // TODO: handle error
                        let _ = message_sender.send(Message::Propose(propose_message)).await;
                    }
                    StateOutput::Prevote {
                        height,
                        round,
                        proposal_hash,
                    } => {
                        let prevote_message =
                            make_prevote_message(height, round, proposal_hash, &consensus_config);
                        let next_state_input = message_to_state_input(
                            Message::Prevote(prevote_message.clone()),
                            &state_machine,
                            &chain_config,
                            &consensus_config,
                        )
                        .expect("failed to convert local message to state input");

                        next_state_outputs.extend(state_machine.step(next_state_input));

                        // TODO: handle error
                        let _ = message_sender.send(Message::Prevote(prevote_message)).await;
                    }
                    StateOutput::Precommit {
                        height,
                        round,
                        proposal_hash,
                    } => {
                        let precommit_message =
                            make_precommit_message(height, round, proposal_hash, &consensus_config);
                        let next_state_input = message_to_state_input(
                            Message::Precommit(precommit_message.clone()),
                            &state_machine,
                            &chain_config,
                            &consensus_config,
                        )
                        .expect("failed to convert local message to state input");

                        next_state_outputs.extend(state_machine.step(next_state_input));

                        // TODO: handle error
                        let _ = message_sender
                            .send(Message::Precommit(precommit_message))
                            .await;
                    }
                    StateOutput::Commit {
                        height, proposal, ..
                    } => {
                        chain_port.commit(height, proposal.into_block());

                        let next_height = height.checked_add(1).expect("height overflow");

                        next_state_outputs.extend(state_machine.step(StateInput::StartHeight {
                            height: next_height,
                        }));
                    }
                    StateOutput::RoundFailure { height, round, .. } => {
                        let next_round = round.checked_add(1).expect("round overflow");

                        next_state_outputs.extend(state_machine.step(StateInput::StartRound {
                            height,
                            round: next_round,
                        }));
                    }
                }
            }

            current_state_outputs = next_state_outputs;
        }
    }
}

fn message_to_state_input(
    message: Message,
    state_machine: &StateMachine<ProposalBlock, ProposalBlockValidator>,
    chain_config: &ChainConfig,
    consensus_config: &ConsensusConfig,
) -> Result<StateInput<ProposalBlock>, MessageVerificationError> {
    message.verify_stateless(
        &state_machine.compute_proposer(),
        chain_config,
        consensus_config,
    )?;

    Ok(match message {
        Message::Propose(msg_propose) => StateInput::ProposalReceived {
            height: msg_propose.payload.height,
            round: msg_propose.payload.round,
            proposal: ProposalBlock::new(msg_propose.payload.proposal),
            justification: msg_propose.payload.justification.map(|justification| {
                Justification::new(
                    justification.height,
                    justification.round,
                    justification
                        .evidences
                        .into_iter()
                        .map(|evidence| Evidence::new(evidence.address, evidence.signature))
                        .collect(),
                )
            }),
        },
        Message::Prevote(msg_prevote) => StateInput::PrevoteReceived {
            height: msg_prevote.payload.height,
            round: msg_prevote.payload.round,
            address: msg_prevote.payload.address,
            proposal_hash: msg_prevote.payload.proposal_hash,
            signature: msg_prevote.signature,
        },
        Message::Precommit(msg_precommit) => StateInput::PrecommitReceived {
            height: msg_precommit.payload.height,
            round: msg_precommit.payload.round,
            address: msg_precommit.payload.address,
            proposal_hash: msg_precommit.payload.proposal_hash,
            signature: msg_precommit.signature,
        },
    })
}

fn make_propose_message(
    height: u128,
    round: u32,
    polka: Option<Polka<ProposalBlock>>,
    block_port: &impl BlockPort,
    consensus_config: &ConsensusConfig,
) -> MsgPropose {
    let (proposal, justification) = match polka {
        Some(polka) => (
            polka.proposal.into_block(),
            Some(ProposeJustificationPayload::new(
                polka.justification.height,
                polka.justification.round,
                polka.proposal_hash,
                polka
                    .justification
                    .evidences
                    .into_iter()
                    .map(|evidence| {
                        ProposeEvidencePayload::new(evidence.address, evidence.signature)
                    })
                    .collect(),
            )),
        ),
        None => (block_port.next_candidate(), None),
    };

    let payload = ProposePayload {
        height,
        round,
        address: consensus_config.address,
        proposal,
        justification,
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPropose { payload, signature }
}

fn make_prevote_message(
    height: u128,
    round: u32,
    proposal_hash: Option<Hash>,
    consensus_config: &ConsensusConfig,
) -> MsgPrevote {
    let payload = PrevotePayload {
        height,
        round,
        address: consensus_config.address,
        proposal_hash,
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPrevote { payload, signature }
}

fn make_precommit_message(
    height: u128,
    round: u32,
    proposal_hash: Option<Hash>,
    consensus_config: &ConsensusConfig,
) -> MsgPrecommit {
    let payload = PrecommitPayload {
        height,
        round,
        address: consensus_config.address,
        proposal_hash,
    };

    let mut out = Vec::with_capacity(payload.signing_payload_len());
    payload.encode_signing_payload(&mut out);
    let signature = sign(&consensus_config.secret_key, &out);

    MsgPrecommit { payload, signature }
}
