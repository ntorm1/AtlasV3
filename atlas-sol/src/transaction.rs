use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_error::ProgramError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::{Message, Transaction};
use std::collections::HashMap;

//https://docs.anza.xyz/runtime/programs/#config-program
static SYS_PROGRAM_ID: &str = "11111111111111111111111111111111";
static CONFIG_PROGRAM_ID: &str = "Config1111111111111111111111111111111111111";
static STAKE_PROGRAM_ID: &str = "Stake11111111111111111111111111111111111111";
static VOTE_PROGRAM_ID: &str = "Vote111111111111111111111111111111111111111";
static ADDRESS_LOOKUP_PROGRAM_ID: &str = "AddressLookupTab1e1111111111111111111111111";
static ED25519_PROGRAM_ID: &str = "Ed25519SigVerify111111111111111111111111111";
static SECP256K1_PROGRAM_ID: &str = "KeccakSecp256k11111111111111111111111111111";
static SECP256R1_PROGRAM_ID: &str = "Secp256r1SigVerify1111111111111111111111111";

//=======================================================================
#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedInstruction {
    pub program_id: Pubkey,
    pub data: Vec<u8>,
    pub keys: Vec<Pubkey>,
}

//=======================================================================
#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedMessage {
    pub instructions: Vec<DecodedInstruction>,
    pub raw_message: Vec<u8>,
}

//=======================================================================
async fn to_instruction(
    connection: &RpcClient,
    wallet_pubkey: &Pubkey,
    account_keys: &[Pubkey],
    instruction: &Instruction,
    instruction_index: usize,
) -> Result<DecodedInstruction, ProgramError> {
    let program_id = instruction.program_id.clone();
    let data = instruction.data.clone();
    let keys = instruction.accounts.clone();
    Ok(DecodedInstruction {
        program_id,
        data,
        keys,
    })
}

//=======================================================================
pub async fn decode_message(
    connection: &RpcClient,
    wallet_pubkey: &Pubkey,
    message: &[u8],
) -> Result<DecodedMessage, ProgramError> {
    let message = Message::deserialize(message)?;
    let instructions = message.instructions;
    let mut decoded_instructions: Vec<DecodedInstruction> = Vec::new();
    for (i, instruction) in instructions.iter().enumerate() {
        // Process each instruction
        let decoded_instruction = to_instruction(
            connection,
            wallet_pubkey,
            &message.account_keys,
            instruction,
            i,
        )
        .await?;
        decoded_instructions.push(decoded_instruction);
    }
    Ok(DecodedMessage {
        instructions: decoded_instructions,
        raw_message: message.serialize()?,
    })
}
