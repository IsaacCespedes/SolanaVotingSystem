use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(Debug)]
struct Voter {
    weight: u32,
    voted: bool,
    vote: u32,
}

impl Voter {
    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        let weight = u32::from_le_bytes(data[..4].try_into().unwrap());
        let voted = data[4] != 0;
        let vote = u32::from_le_bytes(data[5..9].try_into().unwrap());

        Ok(Voter {
            weight,
            voted,
            vote,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.weight.to_le_bytes());
        bytes.push(self.voted as u8);
        bytes.extend_from_slice(&self.vote.to_le_bytes());

        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        let weight = u32::from_le_bytes(bytes[..4].try_into().unwrap());
        let voted = bytes[4] != 0;
        let vote = u32::from_le_bytes(bytes[5..9].try_into().unwrap());

        Ok(Voter {
            weight,
            voted,
            vote,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.weight.to_le_bytes());
        bytes.push(self.voted as u8);
        bytes.extend_from_slice(&self.vote.to_le_bytes());

        bytes
    }
}

#[derive(Debug)]
struct Proposal {
    name: [u8; 32],
    vote_count: u32,
}

impl Proposal {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ProgramError> {
        let name = bytes[..32].try_into().unwrap();
        let vote_count = u32::from_le_bytes(bytes[32..36].try_into().unwrap());

        Ok(Proposal { name, vote_count })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.name);
        bytes.extend_from_slice(&self.vote_count.to_le_bytes());

        bytes
    }

    fn deserialize_list(data: &[u8]) -> Result<Vec<Self>, ProgramError> {
        let mut proposals = Vec::new();
        let mut offset = 4; // Skip the winning proposal index

        while offset < data.len() {
            let name = data[offset..offset + 32].try_into().unwrap();
            let vote_count = u32::from_le_bytes(data[offset + 32..offset + 36].try_into().unwrap());

            proposals.push(Proposal { name, vote_count });

            offset += 36;
        }

        Ok(proposals)
    }
}

#[derive(Debug)]
struct SimpleVotingSystem {
    chairperson: Pubkey,
    voters: Vec<(Pubkey, Voter)>,
    proposals: Vec<Proposal>,
}

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Simple Voting System Rust program entrypoint");

    // Parse the instruction data and call the appropriate function based on its value
    match instruction_data[0] {
        0 => {
            // Give right to vote instruction
            let result = give_right_to_vote(program_id, accounts, instruction_data);

            match result {
                Ok(()) => {
                    // Handle success case
                    println!("Voting rights granted successfully");
                    return Ok(());
                }
                Err(error) => match error {
                    ProgramError::InvalidAccountData => {
                        // Handle specific error case
                        println!("Encountered InvalidAccountData: {:?}", error);
                    }
                    _ => {
                        // Handle any other error case
                        println!("Encountered an unknown error: {:?}", error);
                        return Err(ProgramError::Custom(0));
                    }
                },
            }
        }
        1 => {
            // Vote instruction
            let result = vote(program_id, accounts, instruction_data);

            match result {
                Ok(()) => {
                    // Handle success case
                    println!("Voting rights granted successfully");
                    return Ok(());
                }
                Err(error) => match error {
                    ProgramError::InvalidAccountData => {
                        // Handle specific error case
                        println!("Encountered InvalidAccountData: {:?}", error);
                    }
                    _ => {
                        // Handle any other error case
                        println!("Encountered an unknown error: {:?}", error);
                        return Err(ProgramError::Custom(0));
                    }
                },
            }
        }
        _ => {
            println!("Invalid Instruction");
            return Err(ProgramError::Custom(0));
        }
    }
    Ok(())
}

fn give_right_to_vote(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    //let chairperson_account = next_account_info(accounts_iter)?;
    let voter_account = next_account_info(accounts_iter)?;

    // // Check if the sender is the chairperson
    // if *chairperson_account.key != chairperson_public_key {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    // Check if the voter has already voted
    let mut voter_data = voter_account.data.borrow_mut();
    let voter = Voter::deserialize(&voter_data)?;

    if voter.voted {
        return Err(ProgramError::InvalidAccountData);
    }

    // Give the voter the right to vote
    let voter = Voter {
        weight: 1,
        voted: false,
        vote: 0,
    };
    voter_data.copy_from_slice(&voter.to_bytes());

    Ok(())
}

fn vote(_program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let voter_account = next_account_info(accounts_iter)?;
    let proposal_account = next_account_info(accounts_iter)?;

    // Parse the proposal index from the instruction data
    let proposal_index = u32::from_le_bytes(instruction_data[1..].try_into().unwrap());

    // Retrieve the voter and proposal data
    let voter_data = &mut voter_account.data.borrow_mut();
    let proposal_data = &mut proposal_account.data.borrow_mut();
    let mut voter = Voter::from_bytes(voter_data)?;
    let mut proposal = Proposal::from_bytes(proposal_data)?;

    // Check if the voter has the right to vote
    if voter.weight == 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Check if the voter has already voted
    if voter.voted {
        return Err(ProgramError::InvalidAccountData);
    }

    // Update the voter and proposal data
    voter.voted = true;
    voter.vote = proposal_index;
    proposal.vote_count += voter.weight;

    // Save the updated data back to the accounts
    voter_data.copy_from_slice(&voter.to_bytes());
    proposal_data.copy_from_slice(&proposal.to_bytes());

    Ok(())
}

fn winning_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account = next_account_info(accounts_iter)?;

    let proposal_data = &proposal_account.data.borrow();
    let proposals = Proposal::deserialize_list(proposal_data)?;

    let mut winning_proposal = 0;
    let mut winning_vote_count = 0;

    for (index, proposal) in proposals.iter().enumerate() {
        if proposal.vote_count > winning_vote_count {
            winning_vote_count = proposal.vote_count;
            winning_proposal = index as u32;
        }
    }

    let mut result_data = vec![0u8; 4];
    result_data.copy_from_slice(&winning_proposal.to_le_bytes());
    proposal_account
        .data
        .borrow_mut()
        .copy_from_slice(&result_data);

    Ok(())
}

fn winner_name(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let proposal_account = next_account_info(accounts_iter)?;

    let proposal_data = &proposal_account.data.borrow();
    let proposals = Proposal::deserialize_list(proposal_data)?;

    let winning_proposal = u32::from_le_bytes(proposal_data[..4].try_into().unwrap());

    let winner_name = proposals[winning_proposal as usize].name;

    let mut result_data = vec![0u8; 32];
    result_data.copy_from_slice(&winner_name);
    proposal_account
        .data
        .borrow_mut()
        .copy_from_slice(&result_data);

    Ok(())
}

impl SimpleVotingSystem {
    fn deserialize(data: &[u8]) -> Result<Self, ProgramError> {
        let chairperson = Pubkey::new_from_array(data[..32].try_into().unwrap());

        let mut offset = 32;
        let mut voters = Vec::new();

        while offset < data.len() {
            let voter_key = Pubkey::new_from_array(data[offset..offset + 32].try_into().unwrap());
            let weight = u32::from_le_bytes(data[offset + 32..offset + 36].try_into().unwrap());
            let voted = data[offset + 36] != 0;
            let vote = u32::from_le_bytes(data[offset + 37..offset + 41].try_into().unwrap());

            voters.push((
                voter_key,
                Voter {
                    weight,
                    voted,
                    vote,
                },
            ));

            offset += 41;
        }

        let mut proposals = Vec::new();
        offset += 4; // Skip the winning proposal index

        while offset < data.len() {
            let name = data[offset..offset + 32].try_into().unwrap();
            let vote_count = u32::from_le_bytes(data[offset + 32..offset + 36].try_into().unwrap());

            proposals.push(Proposal { name, vote_count });

            offset += 36;
        }

        Ok(SimpleVotingSystem {
            chairperson,
            voters,
            proposals,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.chairperson.to_bytes());

        for (voter_key, voter) in &self.voters {
            bytes.extend_from_slice(&voter_key.to_bytes());
            bytes.extend_from_slice(&voter.weight.to_le_bytes());
            bytes.push(voter.voted as u8);
            bytes.extend_from_slice(&voter.vote.to_le_bytes());
        }

        bytes.extend_from_slice(&(self.proposals.len() as u32).to_le_bytes());

        for proposal in &self.proposals {
            bytes.extend_from_slice(&proposal.to_bytes());
        }

        bytes
    }
}
