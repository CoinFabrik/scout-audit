#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Map, String,
    Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum URError {
    // Unexpected Revert Error
    AccountAlreadyVoted = 1,
    CandidateAlreadyAdded = 2,
    CandidateDoesntExist = 3,
    Overflow = 4,
    TimestampBeforeCurrentBlock = 5,
    VoteEnded = 6,
}

#[derive(Debug, Clone, PartialEq)]
#[contracttype]
pub struct State {
    total_votes: u64,
    candidates: Vec<Address>,
    votes: Map<Address, u64>,
    already_voted: Map<Address, bool>,
    most_voted_candidate: Address,
    candidate_votes: u64,
    vote_timestamp_end: u64,
}

const STATE: Symbol = symbol_short!("STATE");

#[contract]
pub struct UnexpectedRevert;

#[contractimpl]
impl UnexpectedRevert {
    pub fn init(env: Env, end_timestamp: u64) -> Result<State, URError> {
        if end_timestamp <= env.ledger().timestamp() {
            return Err(URError::TimestampBeforeCurrentBlock);
        }

        let zero_string: String = String::from_str(&env, "00000000000000000000000000000000");
        let zero_addr = Address::from_string(&zero_string); //CHECK
        let state = State {
            total_votes: 0,
            most_voted_candidate: zero_addr,
            candidate_votes: 0,
            candidates: Vec::new(&env),
            already_voted: Map::new(&env),
            votes: Map::new(&env),
            vote_timestamp_end: end_timestamp,
        };

        env.storage().instance().set(&STATE, &state);
        Ok(state)
    }

    pub fn add_candidate(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
        let mut state = Self::get_state(env.clone());
        if Self::vote_ended(env.clone()) {
            return Err(URError::VoteEnded);
        }
        if state.already_voted.contains_key(caller.clone()) {
            Err(URError::AccountAlreadyVoted)
        } else {
            state.candidates.push_back(candidate.clone());
            state.votes.set(candidate, 0);
            Ok(())
        }
    }

    pub fn get_votes_for_a_candidate(env: Env, candidate: Address) -> Result<u64, URError> {
        let state = Self::get_state(env.clone());
        state
            .votes
            .get(candidate)
            .ok_or(URError::CandidateDoesntExist)
    }

    pub fn most_voted_candidate_votes(env: Env) -> u64 {
        let state = Self::get_state(env);
        state.candidate_votes
    }

    pub fn most_voted_candidate(env: Env) -> Address {
        let state = Self::get_state(env);
        state.most_voted_candidate
    }

    pub fn get_total_votes(env: Env) -> u64 {
        let state = Self::get_state(env);
        state.total_votes
    }

    pub fn get_total_candidates(env: Env) -> u64 {
        let state = Self::get_state(env);
        state.candidates.len() as u64
    }

    pub fn get_candidate(env: Env, index: u32) -> Result<Address, URError> {
        let state = Self::get_state(env);
        if index < state.candidates.len() {
            Ok(state.candidates.get(index).unwrap())
        } else {
            Err(URError::CandidateDoesntExist)
        }
    }

    pub fn account_has_voted(env: Env, account: Address) -> bool {
        let state = Self::get_state(env);
        state.already_voted.get(account).unwrap_or(false)
    }

    pub fn vote(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
        caller.require_auth();
        let mut state = Self::get_state(env.clone());
        if Self::vote_ended(env.clone()) {
            return Err(URError::VoteEnded);
        }

        if state.already_voted.contains_key(caller.clone()) {
            Err(URError::AccountAlreadyVoted)
        } else {
            state.already_voted.set(caller, true);
            let votes = state
                .votes
                .get(candidate.clone())
                .ok_or(URError::CandidateDoesntExist)?
                .checked_add(1)
                .ok_or(URError::Overflow)?;
            state.votes.set(candidate.clone(), votes);
            state.total_votes.checked_add(1).ok_or(URError::Overflow)?;
            if state.candidate_votes < votes {
                state.candidate_votes = votes;
                state.most_voted_candidate = candidate;
            }
            Ok(())
        }
    }

    pub fn vote_ended(env: Env) -> bool {
        let state = Self::get_state(env.clone());
        state.vote_timestamp_end <= env.ledger().timestamp()
    }

    pub fn get_state(env: Env) -> State {
        env.storage().instance().get(&STATE).unwrap()
    }
}
