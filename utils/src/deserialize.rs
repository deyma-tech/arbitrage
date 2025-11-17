use solana_program::clock::Slot;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::fmt;

#[derive(Default, Debug, Clone)]
pub struct Messages {
    pub signature: Signature,
    pub message: Vec<Message>,
    pub slot: Slot,
}

#[derive(Default, Debug, Clone)]
pub struct MessagesV2 {
    pub message: Vec<Message>,
    pub slot: Slot,
}

#[derive(Default, Clone)]
pub struct Message {
    pub pubkey: Pubkey,
    pub owner: Pubkey,
    pub data: Vec<u8>,
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Data")
            .field("pubkey", &self.pubkey)
            .field("owner", &self.owner)
            .field("data length", &self.data.len())
            .finish()
    }
}

pub fn deserialize(payload: Vec<u8>) -> anyhow::Result<Messages> {
    let length = payload.len();
    let signature = Signature::try_from(&payload[..64]).map_err(|e| anyhow::format_err!("signature error: {}", e))?;
    let dl = TryInto::<[u8; 4]>::try_into(payload[64..68].to_vec())
        .map_err(|_| anyhow::anyhow!("deserialize: failed to convert data length slice to array"))?;
    let mut data_length = u32::from_le_bytes(dl) as usize;
    let s = TryInto::<[u8; 8]>::try_into(payload[68..76].to_vec())
        .map_err(|_| anyhow::anyhow!("deserialize: failed to convert slot slice to array"))?;
    let slot = Slot::from_le_bytes(s);
    let mut msgs = Messages {
        signature,
        message: vec![],
        slot,
    };
    let mut cursor = 76;
    loop {
        let pubkey = Pubkey::try_from(payload[cursor..cursor + 32].to_vec())
            .map_err(|_| anyhow::anyhow!("deserialize: failed to convert pubkey slice to array"))?;
        cursor += 32;
        let owner = Pubkey::try_from(payload[cursor..cursor + 32].to_vec())
            .map_err(|_| anyhow::anyhow!("deserialize: failed to convert owner slice to array"))?;
        cursor += 32;
        let data = &payload[cursor..cursor + data_length];
        cursor += data_length;
        let msg = Message {
            pubkey,
            owner,
            data: data.to_vec(),
        };
        msgs.message.push(msg);
        // signature 64
        cursor += 64;
        if cursor >= length {
            break;
        }
        data_length =
            //u32::from_le_bytes(payload[cursor..cursor + 4].to_vec().try_into().un_wrap()) as usize;
            u32::from_le_bytes(TryInto::<[u8; 4]>::try_into(payload[cursor..cursor + 4].to_vec()).map_err(|_| anyhow::anyhow!("deserialize: data_length"))?) as usize;
        cursor += 4;
        // slot 8
        cursor += 8;
    }
    // let mut msgs_token = msgs.message.iter().filter(|x| x.owner == spl_token::ID).map(|x| x.clone()).collect::<Vec<_>>();
    // let msgs_non_token = msgs.message.into_iter().filter(|x| x.owner == spl_token::ID).collect::<Vec<_>>();
    // msgs_token.extend(ms)
    Ok(msgs)
}

pub fn deserialize_v2(payload: Vec<u8>) -> anyhow::Result<MessagesV2> {
    let length = payload.len();
    let dl = TryInto::<[u8; 4]>::try_into(payload[..4].to_vec())
        .map_err(|_| anyhow::anyhow!("deserialize: failed to convert data length slice to array"))?;
    let mut data_length = u32::from_le_bytes(dl) as usize;
    let s = TryInto::<[u8; 8]>::try_into(payload[4..12].to_vec())
        .map_err(|_| anyhow::anyhow!("deserialize: failed to convert slot slice to array"))?;
    let slot = Slot::from_le_bytes(s);
    let mut msgs = MessagesV2 { message: vec![], slot };
    let mut cursor = 12;
    loop {
        let pubkey = Pubkey::try_from(payload[cursor..cursor + 32].to_vec())
            .map_err(|_| anyhow::anyhow!("deserialize: failed to convert pubkey slice to array"))?;
        cursor += 32;
        let owner = Pubkey::try_from(payload[cursor..cursor + 32].to_vec())
            .map_err(|_| anyhow::anyhow!("deserialize: failed to convert owner slice to array"))?;
        cursor += 32;
        let data = &payload[cursor..cursor + data_length];
        cursor += data_length;
        let msg = Message {
            pubkey,
            owner,
            data: data.to_vec(),
        };
        msgs.message.push(msg);
        if cursor >= length {
            break;
        }
        data_length = u32::from_le_bytes(
            TryInto::<[u8; 4]>::try_into(payload[cursor..cursor + 4].to_vec())
                .map_err(|_| anyhow::anyhow!("deserialize: data_length"))?,
        ) as usize;
        cursor += 4;
        // slot 8
        cursor += 8;
    }
    // let mut msgs_token = msgs.message.iter().filter(|x| x.owner == spl_token::ID).map(|x| x.clone()).collect::<Vec<_>>();
    // let msgs_non_token = msgs.message.into_iter().filter(|x| x.owner == spl_token::ID).collect::<Vec<_>>();
    // msgs_token.extend(ms)
    Ok(msgs)
}
