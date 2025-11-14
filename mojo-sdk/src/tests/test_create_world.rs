use super::utils::helpers as utils;
use crate::encode_packed;

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        client::{RpcType, SdkClient},
        impl_mojo_state_pod, MojoState,
    };
    use bytemuck::{Pod, Zeroable};
    use solana_program::msg;
    use std::io::Error;

    use crate::types::derive_pda;
    use sha2::{Digest, Sha256};
    use solana_instruction::{AccountMeta, Instruction};
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
    pub fn test_create_world() -> Result<(), Error> {
        let (client, creator) = setup();
        msg!("Creator public key: {}", creator.pubkey());

        let new_world = "New World";
        let starting_position = Position { x: 0, y: 10 };
        let world = client
            .create_world(&creator, new_world, starting_position)
            .map_err(|e| e.to_string())
            .unwrap();

        // FIXME: Error here due to faulty consruction of seeds in world.rs
        msg!("New World: {}", world.world_pda);

        // let mut combined_seeds = Vec::new();
        // combined_seeds.extend_from_slice(new_world.as_bytes());
        // combined_seeds.extend_from_slice(creator.pubkey().as_ref());
        //
        // // Derive the PDA
        // let (world_pda, _bump) = derive_pda(
        //     &[&seed_bytes, creator.pubkey().as_ref()],
        //     &client.program_id,
        // );
        Ok(())
    }
}
