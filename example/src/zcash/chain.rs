use crate::l1::{block::L1BlockInfoImpl, head::L1HeadImpl, tx::DepositTx};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::mpsc::Sender;

pub struct ZcashLayer1 {
    rpc_url: String,
    sender: Sender<L1BlockInfoImpl>,
}

impl ZcashLayer1 {
    pub fn new(rpc_url: String, sender: Sender<L1BlockInfoImpl>) -> Self {
        Self { rpc_url, sender }
    }

    pub fn run(&mut self) {
        let sender = self.sender.clone();
        let rpc_url = self.rpc_url.clone();

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let mut last_height = 0u64;

            loop {
                if let Ok(block) = Self::fetch_block(&client, &rpc_url, &mut last_height).await {
                    let _ = sender.send(block).await;
                }
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
        });
    }

    async fn fetch_block(client: &reqwest::Client, url: &str, last: &mut u64) -> anyhow::Result<L1BlockInfoImpl> {
        let count: serde_json::Value = client.post(url)
            .json(&serde_json::json!({"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}))
            .send().await?.json().await?;

        let height = count["result"].as_u64().ok_or(anyhow::anyhow!("no height"))?;
        if height <= *last { return Err(anyhow::anyhow!("no new block")); }
        *last = height;

        let hash_resp: serde_json::Value = client.post(url)
            .json(&serde_json::json!({"jsonrpc":"2.0","method":"getblockhash","params":[height],"id":1}))
            .send().await?.json().await?;

        let hash_hex = hash_resp["result"].as_str().ok_or(anyhow::anyhow!("no hash"))?;
        let mut hash = [0u8; 32];
        hex::decode_to_slice(hash_hex, &mut hash).ok();

        let block: serde_json::Value = client.post(url)
            .json(&serde_json::json!({"jsonrpc":"2.0","method":"getblock","params":[hash_hex, 1],"id":1}))
            .send().await?.json().await?;

        let timestamp = block["result"]["time"].as_u64().unwrap_or(0);

        Ok(L1BlockInfoImpl {
            l1_head: L1HeadImpl { hash, height, timestamp },
            deposit_txs: vec![],
            batch: None,
        })
    }
}
