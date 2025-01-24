use alloy::{
    signers::{local::PrivateKeySigner, Signer},
    sol,
    sol_types::SolStruct,
};
use alloy_primitives::{
    address, keccak256, Address, Bytes, FixedBytes, PrimitiveSignature, Uint, B256, U256,
};

sol! {
    #[derive(Debug)]
    struct SafeTx {
        address to;
        uint256 value;
        bytes data;
        uint8 operation;
        uint256 safeTxGas;
        uint256 baseGas;
        uint256 gasPrice;
        address gasToken;
        address refundReceiver;
        uint256 nonce;
    }

    struct EIP712Domain {
        uint256 chainId;
        address verifyingContract;
    }
}

sol! {
    #[allow(clippy::too_many_arguments)]
    #[sol(rpc)]
    SafeMultisig,
    "safe_abi.json"
}

pub struct SignatureWithAddress {
    pub signature: PrimitiveSignature,
    pub signer_address: Address,
}

pub fn generate_transaction_hash(safe_address: Address, chain_id: U256, data: SafeTx) -> B256 {
    let mut parts = Vec::new();

    parts.extend(hex::decode("1901").unwrap());

    let domain = EIP712Domain {
        chainId: chain_id,
        verifyingContract: safe_address,
    };

    parts.extend(domain.eip712_hash_struct());

    let type_hash = data.eip712_type_hash().0.to_vec();
    let struct_data = data.eip712_encode_data();
    let encoded_data = [type_hash, struct_data].concat();

    let safe_tx_data_hash = keccak256(encoded_data);

    parts.extend(safe_tx_data_hash);

    keccak256(parts)
}

pub fn create_private_key_signer(private_key: &str) -> PrivateKeySigner {
    PrivateKeySigner::from_bytes(&B256::new(
        hex::decode(private_key).unwrap().try_into().unwrap(),
    ))
    .unwrap()
}

pub async fn sign_hash(
    owner: &PrivateKeySigner,
    tx_hash: &FixedBytes<32>,
) -> anyhow::Result<SignatureWithAddress> {
    let signature = owner.sign_hash(&tx_hash).await?;
    let signer_address = owner.address();
    Ok(SignatureWithAddress {
        signature,
        signer_address,
    })
}

// to be called by sequencer on receiving a signature from the reporter in order to verify it is valid
pub fn verify_message_recovery(
    signature: PrimitiveSignature,
    tx_hash: &FixedBytes<32>,
    signer_address: Address,
) {
    let recovered_address = signature.recover_address_from_prehash(&tx_hash).unwrap();
    assert_eq!(signer_address, recovered_address);
}

pub fn create_safe_tx(contract_address: Address, calldata: Bytes, nonce: Uint<256, 4>) -> SafeTx {
    SafeTx {
        to: contract_address,
        value: U256::from(0),
        data: calldata,
        operation: 0,
        safeTxGas: U256::from(0),
        gasPrice: U256::from(0),
        baseGas: U256::from(0),
        gasToken: address!("0000000000000000000000000000000000000000"),
        refundReceiver: address!("0000000000000000000000000000000000000000"),
        nonce: nonce,
    }
}

pub fn signature_to_bytes(signature: PrimitiveSignature) -> Vec<u8> {
    let v = if signature.v() { 28 } else { 27 };
    let r_bytes: [u8; 32] = signature.r().to_be_bytes();
    let s_bytes: [u8; 32] = signature.s().to_be_bytes();
    let mut signature_bytes = Vec::with_capacity(65);
    signature_bytes.extend_from_slice(&r_bytes);
    signature_bytes.extend_from_slice(&s_bytes);
    signature_bytes.push(v);

    signature_bytes
}

pub async fn get_signature_bytes(
    signatures_with_addresses: &mut Vec<SignatureWithAddress>,
) -> Vec<u8> {
    // Gnosis safe requires signatures to be sorted by signer address
    signatures_with_addresses.sort_by(|a, b| a.signer_address.cmp(&b.signer_address));

    let signature_bytes: Vec<u8> = signatures_with_addresses
        .into_iter()
        .flat_map(|entry| signature_to_bytes(entry.signature))
        .collect();
    signature_bytes
}
