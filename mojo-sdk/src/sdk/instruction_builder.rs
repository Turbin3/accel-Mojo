//! Instruction Builder utilities for underlying solana game engine program

use crate::{
    derive_pda,
    errors::MojoSDKError,
    types::{GenIxHandler, MojoInstructionDiscriminator},
    utils::helpers::compute_hash,
};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_system_program::id as system_program_id;
use solana_sysvar::rent::ID as rent_id;

/// Builder for update_delegated_account instruction
pub struct UpdateDelegatedAccountBuilder {
    program_id: Pubkey,
    creator: Pubkey,
    account_to_update: Pubkey,
    gen_handler: GenIxHandler,
    state_data: Vec<u8>,
}

impl UpdateDelegatedAccountBuilder {
    pub fn new(
        program_id: Pubkey,
        creator: Pubkey,
        account_to_update: Pubkey,
        seed: &Vec<u8>,
        state_data: Vec<u8>,
    ) -> Self {
        let gen_handler = GenIxHandler::new(seed, state_data.len());

        Self {
            program_id,
            creator,
            account_to_update,
            gen_handler,
            state_data,
        }
    }

    pub fn build(self) -> Result<Instruction, MojoSDKError> {
        // Build instruction data: [discriminator][GenIxHandler][game_state_data]
        let mut instruction_data =
            Vec::with_capacity(1 + GenIxHandler::LEN + self.state_data.len());

        instruction_data.push(MojoInstructionDiscriminator::UpdateDelegatedAccount.into());
        instruction_data.extend_from_slice(bytemuck::bytes_of(&self.gen_handler));
        instruction_data.extend_from_slice(&self.state_data);

        // Build accounts
        let accounts = vec![
            AccountMeta::new(self.creator, true), // creator (signer)
            AccountMeta::new(self.account_to_update, false), // account_to_update
            AccountMeta::new(system_program_id(), false),
            AccountMeta::new(rent_id, false),
        ];

        Ok(Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        })
    }
}
