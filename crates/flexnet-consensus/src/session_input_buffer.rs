use crate::{proposal::Proposal, state_input::StateInput};
use std::{cmp::Ordering, collections::HashSet};

fn height_round<P>(state_input: &StateInput<P>) -> (u128, u32)
where
    P: Proposal,
{
    match state_input {
        StateInput::StartHeight { height } => (*height, 0),
        StateInput::StartRound { height, round } => (*height, *round),
        StateInput::ProposalReceived { height, round, .. } => (*height, *round),
        StateInput::PrevoteReceived { height, round, .. } => (*height, *round),
        StateInput::PrecommitReceived { height, round, .. } => (*height, *round),
        StateInput::RoundTimeout { height, round } => (*height, *round),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element<P>
where
    P: Proposal,
{
    state_input: StateInput<P>,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl<P> PartialOrd for Element<P>
where
    P: Proposal,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let (self_height, self_round) = height_round(&self.state_input);
        let (other_height, other_round) = height_round(&other.state_input);

        match self_height.partial_cmp(&other_height) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => {
                return ord;
            }
        }

        match self_round.partial_cmp(&other_round) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => {
                return ord;
            }
        }

        fn state_input_priority<P>(state_input: &StateInput<P>) -> usize
        where
            P: Proposal,
        {
            match state_input {
                StateInput::StartHeight { .. } => 1,
                StateInput::StartRound { .. } => 0,
                StateInput::ProposalReceived { .. } => 4,
                StateInput::PrevoteReceived { .. } => 2,
                StateInput::PrecommitReceived { .. } => 3,
                StateInput::RoundTimeout { .. } => 0,
            }
        }

        let self_priority = state_input_priority(&self.state_input);
        let other_priority = state_input_priority(&other.state_input);

        self_priority
            .partial_cmp(&other_priority)
            .map(|ord| ord.reverse())
    }
}

impl<P> Ord for Element<P>
where
    P: Proposal,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct SessionInputBuffer<P>
where
    P: Proposal,
{
    max_inputs: usize,
    input_set: HashSet<StateInput<P>>,
    ordered_inputs: Vec<Element<P>>,
}

impl<P> SessionInputBuffer<P>
where
    P: Proposal,
{
    pub fn new(max_inputs: usize) -> Self {
        Self {
            max_inputs,
            input_set: HashSet::new(),
            ordered_inputs: Vec::new(),
        }
    }

    pub fn push(&mut self, state_input: StateInput<P>) {
        if self.max_inputs == 0 {
            return;
        }

        if self.input_set.contains(&state_input) {
            return;
        }

        self.input_set.insert(state_input.clone());

        let element = Element { state_input };
        let index = match self.ordered_inputs.binary_search_by(|a| a.cmp(&element)) {
            Ok(index) => index,
            Err(index) => index,
        };
        self.ordered_inputs.insert(index, element);

        while self.ordered_inputs.len() > self.max_inputs {
            let element = match self.ordered_inputs.pop() {
                Some(element) => element,
                None => {
                    break;
                }
            };

            self.input_set.remove(&element.state_input);
        }
    }

    pub fn pop(&mut self, height: u128, round: u32) -> Vec<StateInput<P>> {
        let index = match self.ordered_inputs.iter().position(|element| {
            let (element_height, element_round) = height_round(&element.state_input);
            element_height > height || (element_height == height && element_round >= round)
        }) {
            Some(index) => index,
            None => {
                self.input_set = HashSet::new();
                self.ordered_inputs = Vec::new();
                return Vec::new();
            }
        };

        let mut sliced = self.ordered_inputs.split_off(index);
        std::mem::swap(&mut sliced, &mut self.ordered_inputs);

        for element in &sliced {
            self.input_set.remove(&element.state_input);
        }

        std::mem::drop(sliced);

        let index = match self.ordered_inputs.iter().position(|element| {
            let (element_height, element_round) = height_round(&element.state_input);
            element_height > height || (element_height == height && element_round > round)
        }) {
            Some(index) => index,
            None => {
                self.input_set = HashSet::new();

                let mut targets = Vec::new();
                std::mem::swap(&mut targets, &mut self.ordered_inputs);

                return targets
                    .into_iter()
                    .map(|element| element.state_input)
                    .collect();
            }
        };

        let mut sliced = self.ordered_inputs.split_off(index);
        std::mem::swap(&mut sliced, &mut self.ordered_inputs);

        for element in &sliced {
            self.input_set.remove(&element.state_input);
        }

        sliced
            .into_iter()
            .map(|element| element.state_input)
            .collect()
    }
}
