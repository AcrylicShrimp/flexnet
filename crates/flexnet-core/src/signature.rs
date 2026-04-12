use ed25519_dalek::{Signature, VerifyingKey};

use crate::{codec::encode_transfer_signing_payload, transfer::Transfer};

pub fn verify_transfer_signature(transfer: &Transfer) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(transfer.from.as_bytes()) else {
        return false;
    };
    let signature = Signature::from_bytes(&transfer.signature);
    let payload = encode_transfer_signing_payload(transfer);

    verifying_key.verify_strict(&payload, &signature).is_ok()
}
