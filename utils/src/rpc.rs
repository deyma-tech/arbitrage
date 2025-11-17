use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[inline(always)]
pub async fn get_account_data(rpc_client: &RpcClient, account: &Pubkey) -> anyhow::Result<Vec<u8>> {
    let account_data = rpc_client
        .get_account_data(account)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get account {}: {}", account, e))?;
    Ok(account_data)
}

#[inline(always)]
pub async fn get_token_account_amount(rpc_client: &RpcClient, token_account: &Pubkey) -> anyhow::Result<u64> {
    let account_data = get_account_data(rpc_client, token_account).await?;
    let amount = u64::from_le_bytes(account_data[64..72].try_into()?);
    Ok(amount)
}
