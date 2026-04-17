use crate::{
    consensus_config::ConsensusConfig,
    consensus_driver::{
        proposal_block::ProposalBlock, proposal_block_validator::ProposalBlockValidator,
    },
    justification::{Evidence, Justification},
    message::{Message, MessageVerificationError},
    state_input::StateInput,
    state_machine::StateMachine,
};
use flexnet_chain::chain_config::ChainConfig;

pub fn message_to_state_input(
    message: Message,
    state_machine: &StateMachine<ProposalBlock, ProposalBlockValidator>,
    chain_config: &ChainConfig,
    consensus_config: &ConsensusConfig,
) -> Result<StateInput<ProposalBlock>, MessageVerificationError> {
    match message.verify_stateless(
        &state_machine.compute_proposer(),
        chain_config,
        consensus_config,
    ) {
        Ok(_) => {}
        Err(err) => {
            println!("Message verification failed: {:?}", err);
            return Err(err);
        }
    };

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
