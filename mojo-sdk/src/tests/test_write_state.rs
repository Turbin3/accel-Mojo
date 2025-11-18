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
    use solana_keypair::Keypair;
    use solana_pubkey::Pubkey;
    use solana_signer::{EncodableKey, Signer};

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
    pub fn test_write_state() -> Result<(), Error> {
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

        // Create: A new Player state is created because it does not exist yet
        // create a delegated state account manually via CreateAccount instruction
        let state_name = "player_position";
        let player_state = Position { x: 8, y: 9 };

        // Use write_state
        client
            .write_state(&world, state_name, &creator, player_state)
            .map_err(|e| e.to_string())
            .unwrap();

        let fetched_player_state: Position = client
            .read_delegated_state(&world, state_name, &creator_pubkey)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(
            fetched_player_state, player_state,
            "Player state is not created successfully as expected"
        );

        // Rewrite: Give player a new position
        let new_player_state = Position { x: 8, y: 19 };

        // Use write_state
        client
            .write_state(&world, state_name, &creator, new_player_state)
            .map_err(|e| e.to_string())
            .unwrap();

        let fetched_player_state: Position = client
            .read_delegated_state(&world, state_name, &creator_pubkey)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(
            fetched_player_state, new_player_state,
            "Player state is not overridden successfully as expected"
        );

        Ok(())
    }
}
