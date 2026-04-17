use crate::{
    consensus_config::ConsensusConfig,
    consensus_driver::{
        make_messages::{make_propose_message_using_block, make_propose_message_using_polka},
        proposal_block::ProposalBlock,
    },
    messages::msg_propose::MsgPropose,
    polka::Polka,
    ports::block_port::BlockPort,
};
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

pub struct ProposalGenerator<B>
where
    B: BlockPort,
{
    block_port: Arc<Mutex<B>>,
    proposal_sender: Sender<MsgPropose>,
    consensus_config: Arc<ConsensusConfig>,
}

impl<B> ProposalGenerator<B>
where
    B: BlockPort,
{
    pub fn new(
        block_port: B,
        consensus_config: Arc<ConsensusConfig>,
    ) -> (Self, Receiver<MsgPropose>) {
        let (proposal_sender, proposal_receiver) = tokio::sync::mpsc::channel(1);
        let proposal_generator = Self {
            block_port: Arc::new(Mutex::new(block_port)),
            proposal_sender,
            consensus_config,
        };

        (proposal_generator, proposal_receiver)
    }

    pub fn request_proposal(&self, height: u128, round: u32, polka: Option<Polka<ProposalBlock>>) {
        match polka {
            Some(polka) => {
                let proposal_sender = self.proposal_sender.clone();
                let consensus_config = self.consensus_config.clone();

                tokio::spawn(async move {
                    let proposal =
                        generate_proposal_using_polka(height, round, polka, &consensus_config)
                            .await;
                    let _ = proposal_sender.send(proposal).await;
                });
            }
            None => {
                let block_port = self.block_port.clone();
                let proposal_sender = self.proposal_sender.clone();
                let consensus_config = self.consensus_config.clone();

                tokio::spawn(async move {
                    let proposal =
                        generate_proposal_using_block(block_port, height, round, &consensus_config)
                            .await;
                    let proposal = match proposal {
                        Some(proposal) => proposal,
                        None => {
                            return;
                        }
                    };

                    let _ = proposal_sender.send(proposal).await;
                });
            }
        }
    }
}

async fn generate_proposal_using_polka(
    height: u128,
    round: u32,
    polka: Polka<ProposalBlock>,
    consensus_config: &ConsensusConfig,
) -> MsgPropose {
    make_propose_message_using_polka(height, round, polka, consensus_config)
}

async fn generate_proposal_using_block<B>(
    block_port: Arc<Mutex<B>>,
    height: u128,
    round: u32,
    consensus_config: &ConsensusConfig,
) -> Option<MsgPropose>
where
    B: BlockPort,
{
    let mut block_port = block_port.lock().await;
    let block = block_port.next_candidate(height).await?;

    Some(make_propose_message_using_block(
        height,
        round,
        block,
        consensus_config,
    ))
}
