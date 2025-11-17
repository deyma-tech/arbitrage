use crate::base64::b64_encode;
use solana_sdk::transaction::VersionedTransaction;

pub fn check_transaction_size(txn: &VersionedTransaction) -> anyhow::Result<Vec<u8>> {
    let buf = bincode::serialize(&txn)?;
    let size = buf.len();
    if size > 1232 {
        return Err(anyhow::format_err!("Transaction size too large: {}", size));
    }
    Ok(buf)
}

pub const TEMPLATE: &str = r#"{"method":"sendBundle","params": [["{1}"], "{2}"], "id":1, "jsonrpc":"2.0"}"#;

pub const TEMPLATE_V2: &str = r#"{"method":"sendBundle","params": [["{1}", "{2}"], "{3}"], "id":1, "jsonrpc":"2.0"}"#;

// TODO: perf: use a more efficient way to generate the body
//       prio: low - this is not a bottleneck
#[inline]
pub fn generate_body_for_jito_from_data(buffer: &[u8], jito_short: &str) -> anyhow::Result<String> {
    let tx_58 = b64_encode(buffer);
    let msg = TEMPLATE.replace("{1}", &tx_58).replace("{2}", jito_short);
    Ok(msg)
}

#[inline]
pub fn generate_body_for_jito_from_data_v2(buffer0: &[u8], buffer1: &[u8], jito_short: &str) -> anyhow::Result<String> {
    let tx_58 = b64_encode(buffer0);
    let tx_58_2 = b64_encode(buffer1);
    let msg = TEMPLATE_V2
        .replace("{1}", &tx_58)
        .replace("{2}", &tx_58_2)
        .replace("{3}", jito_short);
    Ok(msg)
}

pub fn generate_body_for_jito(txn: &VersionedTransaction, jito_short: &str) -> anyhow::Result<String> {
    let buffer = bincode::serialize(&txn);
    let buffer = buffer.map_err(|e| anyhow::format_err!("InvalidTransaction: {}", e))?;
    generate_body_for_jito_from_data(&buffer, jito_short)
}
