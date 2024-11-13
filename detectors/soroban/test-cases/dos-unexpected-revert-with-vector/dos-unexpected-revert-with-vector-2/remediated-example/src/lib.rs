#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

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
    total_candidates: u32,
    most_voted_candidate: Address,
    candidate_votes: u64,
    vote_timestamp_end: u64,
}

#[derive(Debug, Clone, PartialEq)]
#[contracttype]
pub struct Candidate {
    votes: u64,
}

#[derive(Default)]
#[contracttype]
pub struct AlreadyVoted {
    voted: bool,
}

#[contracttype]
pub enum DataKey {
    State,
    AlreadyVoted(Address),
    Candidate(Address),
}

#[contract]
pub struct UnexpectedRevert;

#[contractimpl]
impl UnexpectedRevert {
    pub fn set_candidate(env: Env, candidate: Address, votes: u64) {
        let cand = Candidate { votes };

        env.storage()
            .instance()
            .set(&DataKey::Candidate(candidate), &cand);
    }

    pub fn retrieve_candidate(env: Env, candidate: Address) -> Result<Candidate, URError> {
        env.storage()
            .instance()
            .get(&DataKey::Candidate(candidate))
            .unwrap_or(Err(URError::CandidateDoesntExist))
    }

    pub fn init(env: Env, end_timestamp: u64) -> Result<State, URError> {
        if end_timestamp <= env.ledger().timestamp() {
            return Err(URError::TimestampBeforeCurrentBlock);
        }

        let zero_string: String = String::from_str(&env, "00000000000000000000000000000000");
        let zero_addr = Address::from_string(&zero_string); // Whenever this is zero address it will mean no candidate has yet been more voted
        let state = State {
            total_votes: 0,
            total_candidates: 0,
            most_voted_candidate: zero_addr,
            candidate_votes: 0,
            vote_timestamp_end: end_timestamp,
        };

        env.storage().instance().set(&DataKey::State, &state);
        Ok(state)
    }

    pub fn add_candidate(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
        let mut state = Self::get_state(env.clone());
        if Self::vote_ended(env.clone()) {
            return Err(URError::VoteEnded);
        }
        if Self::account_has_voted(env.clone(), caller.clone()) {
            Err(URError::AccountAlreadyVoted)
        } else {
            env.storage().instance().set(
                &DataKey::Candidate(candidate.clone()),
                &Candidate { votes: 0 },
            );
            state.total_candidates += 1;
            env.storage().instance().set(&DataKey::State, &state);
            Ok(())
        }
    }

    pub fn account_has_voted(env: Env, caller: Address) -> bool {
        let already_voted: AlreadyVoted = env
            .storage()
            .instance()
            .get(&DataKey::AlreadyVoted(caller))
            .unwrap_or_default();
        already_voted.voted
    }

    pub fn get_votes_for_a_candidate(env: Env, candidate: Address) -> Result<u64, URError> {
        let result: Option<Candidate> =
            env.storage().instance().get(&DataKey::Candidate(candidate));
        match result {
            Some(cand) => Ok(cand.votes),
            None => Err(URError::CandidateDoesntExist),
        }
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

    pub fn get_total_candidates(env: Env) -> u32 {
        let state = Self::get_state(env);
        state.total_candidates
    }

    pub fn get_candidate(env: Env, addr: Address) -> Result<Candidate, URError> {
        let result: Option<Candidate> = env.storage().instance().get(&DataKey::Candidate(addr));
        match result {
            Some(cand) => Ok(cand),
            None => Err(URError::CandidateDoesntExist),
        }
    }

    pub fn vote(env: Env, candidate: Address, caller: Address) -> Result<(), URError> {
        caller.require_auth();
        let mut state = Self::get_state(env.clone());
        if Self::vote_ended(env.clone()) {
            return Err(URError::VoteEnded);
        }

        if Self::account_has_voted(env.clone(), caller.clone()) {
            Err(URError::AccountAlreadyVoted)
        } else {
            env.storage().instance().set(
                &DataKey::AlreadyVoted(caller.clone()),
                &AlreadyVoted { voted: true },
            );
            let votes = Self::get_candidate(env.clone(), candidate.clone())?
                .votes
                .checked_add(1)
                .ok_or(URError::Overflow)?;
            env.storage()
                .instance()
                .set(&DataKey::Candidate(candidate.clone()), &Candidate { votes });
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
        env.storage().instance().get(&DataKey::State).unwrap()
    }
}
