use super::utils::helpers as utils;
use crate::{encode_packed, GenIxHandler, MojoInstructionDiscriminator};

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        client::{RpcType, SdkClient},
        impl_mojo_state_pod, MojoState,
    };
    use bytemuck::{self, Pod, Zeroable};
    use solana_program::msg;
    use std::{
        io::Error,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::types::derive_pda;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_pubkey::Pubkey;
    use solana_signer::{EncodableKey, Signer};
    use solana_system_program;
    use solana_sysvar::rent::ID as RENT_ID;
    use solana_transaction::Transaction;

    // use crate::instructions::MojoInstructions::CreateAccount;

    const PROGRAM_ID: Pubkey = Pubkey::new_from_array(mojo_program::ID);
    const LAMPORTS_PER_SOL: u64 = 10 ^ 9;

    fn program_id() -> Pubkey {
        PROGRAM_ID
    }

    #[repr(C)]
    #[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
    pub struct Position {
        x: u64,
        y: u64,
    }
    impl_mojo_state_pod!(Position);

    fn setup() -> (SdkClient, Keypair) {
        let client = SdkClient::new(RpcType::Dev);
        let creator =
            Keypair::read_from_file("../dev_wallet.json").expect("Could not read keypair file");
        return (client, creator);
    }

    /**
     * Tests assume use of a funded key
     */
    #[test]
    pub fn test_create_world() -> Result<(), Error> {
        let (client, creator) = setup();
        let creator_pubkey = creator.pubkey();
        msg!("Creator public key: {}", creator_pubkey);

        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let new_world = format!("New World {}", millis);
        let starting_position = Position { x: 0, y: 10 };
        let world = client
            .create_world(&creator, &new_world, starting_position)
            .map_err(|e| e.to_string())
            .unwrap();

        msg!("New World: {}", world.world_pda);

        let world_seed_input =
            encode_packed!(b"world", new_world.as_bytes(), creator_pubkey.as_ref());
        let seed_bytes = utils::compute_hash(&world_seed_input);
        let (expected_pda, _bump) =
            derive_pda(&[&seed_bytes, creator_pubkey.as_ref()], &client.program_id);
        assert_eq!(world.world_pda, expected_pda);

        let fetched_world_state: Position = client
            .read_world(&world)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(fetched_world_state, starting_position);

        // create a delegated state account manually via CreateAccount instruction
        let state_name = "player_position";
        let player_state = Position { x: 9, y: 9 };
        let state_seed_input = crate::encode_packed!(
            b"state",
            world.world_seed_hash.as_ref(),
            state_name.as_bytes(),
            creator_pubkey.as_ref()
        );
        let state_seed_hash = utils::compute_hash(&state_seed_input);
        let (state_pda, _bump) = derive_pda(
            &[&state_seed_hash, creator_pubkey.as_ref()],
            &client.program_id,
        );

        let player_state_bytes = player_state.serialize().unwrap();
        let gen_ix = GenIxHandler::new(&state_seed_input, player_state_bytes.len());
        let create_state_ix = Instruction {
            program_id: client.program_id,
            accounts: vec![
                AccountMeta::new(creator_pubkey, true),
                AccountMeta::new(state_pda, false),
                AccountMeta::new(solana_system_program::id(), false),
                AccountMeta::new(Pubkey::new_from_array(RENT_ID.to_bytes()), false),
            ],
            data: [
                vec![MojoInstructionDiscriminator::CreateAccount as u8],
                bytemuck::bytes_of(&gen_ix).to_vec(),
                player_state_bytes.clone(),
            ]
            .concat(),
        };

        let recent_blockhash = client.client.get_latest_blockhash().unwrap();
        let message = Message::new(&[create_state_ix], Some(&creator_pubkey));
        let transaction = Transaction::new(&[&creator], message, recent_blockhash);
        client
            .client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to create delegated state");

        let fetched_player_state: Position = client
            .read_delegated_state(&world, state_name, &creator_pubkey)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(fetched_player_state, player_state);
        Ok(())
    }
}
