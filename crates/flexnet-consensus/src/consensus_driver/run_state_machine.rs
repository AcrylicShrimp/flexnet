use crate::{
    consensus_config::ConsensusConfig,
    consensus_driver::{
        make_messages::{make_precommit_message, make_prevote_message},
        message_to_state_input::message_to_state_input,
        proposal_block::ProposalBlock,
        proposal_block_validator::ProposalBlockValidator,
        proposal_generator::ProposalGenerator,
        timeout::Timeout,
    },
    message::Message,
    ports::{block_port::BlockPort, chain_port::ChainPort, message_port::MessagePort},
    state_input::StateInput,
    state_machine::StateMachine,
    state_output::StateOutput,
};
use flexnet_chain::chain_config::ChainConfig;
use std::time::Duration;
use tokio::time::Instant;

pub struct StateMachineExecutionContext<'a, M, B, C>
where
    M: MessagePort,
    B: BlockPort,
    C: ChainPort,
{
    pub chain_config: &'a ChainConfig,
    pub consensus_config: &'a ConsensusConfig,
    pub timeout: &'a mut Option<Timeout>,
    pub message_port: &'a mut M,
    pub proposal_generator: &'a ProposalGenerator<B>,
    pub chain_port: &'a mut C,
}

pub fn run_state_machine<'a, M, B, C>(
    input: StateInput<ProposalBlock>,
    context: StateMachineExecutionContext<'a, M, B, C>,
    state_machine: &mut StateMachine<ProposalBlock, ProposalBlockValidator>,
) where
    M: MessagePort,
    B: BlockPort,
    C: ChainPort,
{
    println!("input: {:#?}", input);

    let mut outputs = state_machine.step(input);

    println!("outputs: {:#?}", outputs);

    while !outputs.is_empty() {
        let mut next_outputs = vec![];

        for output in outputs {
            match output {
                StateOutput::StartTimeout {
                    height,
                    round,
                    timeout_ms,
                } => {
                    *context.timeout = Some(Timeout {
                        height,
                        round,
                        instant: Instant::now() + Duration::from_millis(timeout_ms),
                    });
                }
                StateOutput::StartRound { height, round } => {
                    next_outputs
                        .extend(state_machine.step(StateInput::StartRound { height, round }));
                }
                StateOutput::Propose {
                    height,
                    round,
                    polka,
                } => {
                    context
                        .proposal_generator
                        .request_proposal(height, round, polka);
                }
                StateOutput::Prevote {
                    height,
                    round,
                    proposal_hash,
                } => {
                    let prevote = make_prevote_message(
                        height,
                        round,
                        proposal_hash,
                        context.consensus_config,
                    );
                    let next_input = message_to_state_input(
                        Message::Prevote(prevote.clone()),
                        state_machine,
                        context.chain_config,
                        context.consensus_config,
                    )
                    .expect("failed to convert local message to state input");

                    next_outputs.extend(state_machine.step(next_input));

                    let sender = context.message_port.sender().clone();

                    tokio::spawn(async move {
                        let _ = sender.send(Message::Prevote(prevote)).await;
                    });
                }
                StateOutput::Precommit {
                    height,
                    round,
                    proposal_hash,
                } => {
                    let precommit = make_precommit_message(
                        height,
                        round,
                        proposal_hash,
                        context.consensus_config,
                    );
                    let next_input = message_to_state_input(
                        Message::Precommit(precommit.clone()),
                        state_machine,
                        context.chain_config,
                        context.consensus_config,
                    )
                    .expect("failed to convert local message to state input");

                    next_outputs.extend(state_machine.step(next_input));

                    let sender = context.message_port.sender().clone();

                    tokio::spawn(async move {
                        let _ = sender.send(Message::Precommit(precommit)).await;
                    });
                }
                StateOutput::Commit {
                    height,
                    round: _,
                    proposal,
                } => {
                    context.chain_port.commit(height, proposal.into_block());

                    let next_height = height.checked_add(1).expect("height overflow");

                    next_outputs.extend(state_machine.step(StateInput::StartHeight {
                        height: next_height,
                    }));
                }
                StateOutput::RoundFailure {
                    height,
                    round,
                    reason: _,
                } => {
                    let next_round = round.checked_add(1).expect("round overflow");

                    next_outputs.extend(state_machine.step(StateInput::StartRound {
                        height,
                        round: next_round,
                    }));
                }
            }
        }

        println!("next_outputs: {:#?}", next_outputs);

        outputs = next_outputs;
    }
}
