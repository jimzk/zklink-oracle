use pairing::Engine;
use sync_vm::{
    circuit_structures::byte::Byte,
    franklin_crypto::bellman::{plonk::better_better_cs::cs::ConstraintSystem, SynthesisError},
    traits::CSAllocatable,
};

use crate::{
    gadgets::{ecdsa::Signature, keccak160},
    params::NUM_WORMHOLE_SIGNATURES,
    utils::new_synthesis_error,
};

// Circuit representation of [`wormhole vaa`](https://docs.wormhole.com/wormhole/explore-wormhole/vaa)
// We only put part of the VAA fields here.
//
// Representation of wormhole VAA. We only put parts of VAA fields here.
// - https://docs.wormhole.com/wormhole/explore-wormhole/vaa
// - https://github.com/wormhole-foundation/wormhole/blob/bfd4ba40ef2d213ad69bac638c72009ba4a07878/sdk/rust/core/src/vaa.rs#L84-L100
#[derive(Debug, Clone)]
pub struct WormholeMessage<E: Engine> {
    pub signatures: [Signature<E>; NUM_WORMHOLE_SIGNATURES],
    pub body: WormholeBody<E>,
}

impl<E: Engine> WormholeMessage<E> {
    pub fn alloc_from_witness<CS: ConstraintSystem<E>>(
        cs: &mut CS,
        message: wormhole_sdk::Vaa<&serde_wormhole::RawMessage>,
    ) -> Result<Self, SynthesisError> {
        let (header, body): (wormhole_sdk::vaa::Header, wormhole_sdk::vaa::Body<_>) =
            message.into();
        let body = WormholeBody::alloc_from_witness(cs, body)?;
        if header.signatures.len() < NUM_WORMHOLE_SIGNATURES {
            return Err(new_synthesis_error(format!(
                "Only have {} signature. expect {} at least",
                header.signatures.len(),
                NUM_WORMHOLE_SIGNATURES
            )));
        }

        let signatures = (0..NUM_WORMHOLE_SIGNATURES)
            .into_iter()
            .map(|i| {
                let signature = header.signatures[i].signature;
                Signature::alloc_from_bytes_witness(cs, &signature)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            signatures: signatures.try_into().unwrap(),
            body,
        })
    }
}

const LEN_WORMHOLE_BODY_TIMESTAMP: usize = 4;
const LEN_WORMHOLE_BODY_NONCE: usize = 4;
const LEN_WORMHOLE_BODY_EMITTER_CHAIN: usize = 2;
const LEN_WORMHOLE_BODY_EMITTER_ADDRESS: usize = 32;
const LEN_WORMHOLE_BODY_SEQUENCE: usize = 8;
const LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL: usize = 1;
const LEN_WORMHOLE_BODY: usize = LEN_WORMHOLE_BODY_TIMESTAMP
    + LEN_WORMHOLE_BODY_NONCE
    + LEN_WORMHOLE_BODY_EMITTER_CHAIN
    + LEN_WORMHOLE_BODY_EMITTER_ADDRESS
    + LEN_WORMHOLE_BODY_SEQUENCE
    + LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL
    + LEN_MESSAGE;
#[derive(Debug, Clone)]
pub struct WormholeBody<E: Engine> {
    pub timestamp: [Byte<E>; LEN_WORMHOLE_BODY_TIMESTAMP],
    pub nonce: [Byte<E>; LEN_WORMHOLE_BODY_NONCE],
    pub emitter_chain: [Byte<E>; LEN_WORMHOLE_BODY_EMITTER_CHAIN],
    pub emitter_address: [Byte<E>; LEN_WORMHOLE_BODY_EMITTER_ADDRESS],
    pub sequence: [Byte<E>; LEN_WORMHOLE_BODY_SEQUENCE],
    pub consistency_level: [Byte<E>; LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL],
    pub payload: WormholePayload<E>,
}

// Circuit representation of body in wormhole VAA.
// - https://docs.wormhole.com/wormhole/explore-wormhole/vaa#body
// - https://github.com/wormhole-foundation/wormhole/blob/bfd4ba40ef2d213ad69bac638c72009ba4a07878/sdk/rust/core/src/vaa.rs#L112-L121
impl<E: Engine> WormholeBody<E> {
    pub fn new(bytes: [Byte<E>; LEN_WORMHOLE_BODY]) -> Self {
        let mut offset = 0;
        let timestamp = bytes[offset..offset + LEN_WORMHOLE_BODY_TIMESTAMP]
            .try_into()
            .unwrap();
        offset += LEN_WORMHOLE_BODY_TIMESTAMP;
        let nonce = bytes[offset..offset + LEN_WORMHOLE_BODY_NONCE]
            .try_into()
            .unwrap();
        offset += LEN_WORMHOLE_BODY_NONCE;
        let emitter_chain = bytes[offset..offset + LEN_WORMHOLE_BODY_EMITTER_CHAIN]
            .try_into()
            .unwrap();
        offset += LEN_WORMHOLE_BODY_EMITTER_CHAIN;
        let emitter_address = bytes[offset..offset + LEN_WORMHOLE_BODY_EMITTER_ADDRESS]
            .try_into()
            .unwrap();
        offset += LEN_WORMHOLE_BODY_EMITTER_ADDRESS;
        let sequence = bytes[offset..offset + LEN_WORMHOLE_BODY_SEQUENCE]
            .try_into()
            .unwrap();
        offset += LEN_WORMHOLE_BODY_SEQUENCE;
        let consistency_level = bytes[offset..offset + LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL]
            .try_into()
            .unwrap();
        offset += LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL;
        let payload = WormholePayload::new(bytes[offset..offset + LEN_MESSAGE].try_into().unwrap());
        Self {
            timestamp,
            nonce,
            emitter_chain,
            emitter_address,
            sequence,
            consistency_level,
            payload,
        }
    }

    pub fn new_from_slice(bytes: &[Byte<E>]) -> Result<Self, SynthesisError> {
        if bytes.len() != LEN_WORMHOLE_BODY {
            return Err(new_synthesis_error(format!(
                "invalid bytes length {}, expect {}",
                bytes.len(),
                LEN_MESSAGE
            )));
        }
        Ok(Self::new(bytes.try_into().unwrap()))
    }

    pub fn to_bytes(&self) -> [Byte<E>; LEN_WORMHOLE_BODY] {
        let mut bytes = [Byte::<E>::zero(); LEN_WORMHOLE_BODY];
        let mut offset = 0;
        bytes[offset..offset + LEN_WORMHOLE_BODY_TIMESTAMP].copy_from_slice(&self.timestamp);
        offset += LEN_WORMHOLE_BODY_TIMESTAMP;
        bytes[offset..offset + LEN_WORMHOLE_BODY_NONCE].copy_from_slice(&self.nonce);
        offset += LEN_WORMHOLE_BODY_NONCE;
        bytes[offset..offset + LEN_WORMHOLE_BODY_EMITTER_CHAIN]
            .copy_from_slice(&self.emitter_chain);
        offset += LEN_WORMHOLE_BODY_EMITTER_CHAIN;
        bytes[offset..offset + LEN_WORMHOLE_BODY_EMITTER_ADDRESS]
            .copy_from_slice(&self.emitter_address);
        offset += LEN_WORMHOLE_BODY_EMITTER_ADDRESS;
        bytes[offset..offset + LEN_WORMHOLE_BODY_SEQUENCE].copy_from_slice(&self.sequence);
        offset += LEN_WORMHOLE_BODY_SEQUENCE;
        bytes[offset..offset + LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL]
            .copy_from_slice(&self.consistency_level);
        offset += LEN_WORMHOLE_BODY_CONSISTENCY_LEVEL;
        bytes[offset..offset + LEN_MESSAGE].copy_from_slice(&self.payload.to_bytes());
        bytes
    }

    pub fn alloc_from_witness<CS: ConstraintSystem<E>>(
        cs: &mut CS,
        witness: wormhole_sdk::vaa::Body<&serde_wormhole::RawMessage>,
    ) -> Result<Self, SynthesisError> {
        let timestamp = {
            let bytes = witness.timestamp.to_be_bytes();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let nonce = {
            let bytes = witness.nonce.to_be_bytes();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let emitter_chain = {
            let bytes = serde_wormhole::to_vec(&witness.emitter_chain)
                .unwrap()
                .try_into()
                .unwrap();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let emitter_address = {
            let bytes = serde_wormhole::to_vec(&witness.emitter_address)
                .unwrap()
                .try_into()
                .unwrap();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let sequence = {
            let bytes = witness.sequence.to_be_bytes();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let consistency_level = {
            let bytes = witness.consistency_level.to_be_bytes();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let payload = {
            let payload =
                pythnet_sdk::wire::v1::WormholeMessage::try_from_bytes(witness.payload.as_ref())
                    .map_err(new_synthesis_error)?;
            WormholePayload::alloc_from_witness(cs, payload)?
        };
        Ok(Self {
            timestamp,
            nonce,
            emitter_chain,
            emitter_address,
            sequence,
            consistency_level,
            payload,
        })
    }
}

const LEN_MAGIC: usize = 4;
const LEN_PAYLOAD_TYPE: usize = 1;
const LEN_SLOT: usize = 8;
const LEN_RING_SIZE: usize = 4;
const LEN_ROOT: usize = keccak160::WIDTH_HASH_BYTES;
const LEN_MESSAGE: usize = LEN_MAGIC + LEN_PAYLOAD_TYPE + LEN_SLOT + LEN_RING_SIZE + LEN_ROOT;
// Representation of pyth-defined wormhole payload
// - https://github.com/pyth-network/pyth-crosschain/blob/1d82f92d80598e689f4130983d06b12412b83427/pythnet/pythnet_sdk/src/wire.rs#L109-L112
const PAYLOAD_TYPE: u8 = 0; // Fixed payload type for now.
#[derive(Debug, Clone)]
pub struct WormholePayload<E: Engine> {
    pub magic: [Byte<E>; LEN_MAGIC],
    pub payload_type: [Byte<E>; LEN_PAYLOAD_TYPE],
    pub slot: [Byte<E>; LEN_SLOT],
    pub ring_size: [Byte<E>; LEN_RING_SIZE],
    pub root: keccak160::Hash<E>,
}

impl<E: Engine> WormholePayload<E> {
    pub fn new(bytes: [Byte<E>; LEN_MESSAGE]) -> Self {
        let mut offset = 0;
        let magic = bytes[offset..offset + LEN_MAGIC].try_into().unwrap();
        offset += LEN_MAGIC;
        let payload_type = bytes[offset..offset + LEN_PAYLOAD_TYPE].try_into().unwrap();
        offset += LEN_PAYLOAD_TYPE;
        let slot = bytes[offset..offset + LEN_SLOT].try_into().unwrap();
        offset += LEN_SLOT;
        let ring_size = bytes[offset..offset + LEN_RING_SIZE].try_into().unwrap();
        offset += LEN_RING_SIZE;
        let root = bytes[offset..offset + LEN_ROOT].try_into().unwrap();
        Self {
            magic,
            payload_type,
            slot,
            ring_size,
            root,
        }
    }
    pub fn new_from_slice(bytes: &[Byte<E>]) -> Result<Self, SynthesisError> {
        if bytes.len() != LEN_MESSAGE {
            return Err(new_synthesis_error(format!(
                "invalid bytes length {}, expect {}",
                bytes.len(),
                LEN_MESSAGE
            )));
        }
        Ok(Self::new(bytes.try_into().unwrap()))
    }

    pub fn to_bytes(&self) -> [Byte<E>; LEN_MESSAGE] {
        let mut bytes = [Byte::<E>::zero(); LEN_MESSAGE];
        let mut offset = 0;
        bytes[offset..offset + LEN_MAGIC].copy_from_slice(&self.magic);
        offset += LEN_MAGIC;
        bytes[offset..offset + LEN_PAYLOAD_TYPE].copy_from_slice(&self.payload_type);
        offset += LEN_PAYLOAD_TYPE;
        bytes[offset..offset + LEN_SLOT].copy_from_slice(&self.slot);
        offset += LEN_SLOT;
        bytes[offset..offset + LEN_RING_SIZE].copy_from_slice(&self.ring_size);
        offset += LEN_RING_SIZE;
        bytes[offset..offset + LEN_ROOT].copy_from_slice(&self.root);
        bytes
    }

    pub fn alloc_from_witness<CS: ConstraintSystem<E>>(
        cs: &mut CS,
        message: pythnet_sdk::wire::v1::WormholeMessage,
    ) -> Result<Self, SynthesisError> {
        let magic = CSAllocatable::alloc_from_witness(cs, Some(message.magic))?;
        let payload_type = CSAllocatable::alloc_from_witness(cs, Some([PAYLOAD_TYPE]))?;
        let pythnet_sdk::wire::v1::WormholePayload::Merkle(payload) = message.payload;
        let slot = {
            let bytes = payload.slot.to_be_bytes();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let ring_size = {
            let bytes = payload.ring_size.to_be_bytes();
            CSAllocatable::alloc_from_witness(cs, Some(bytes))?
        };
        let root = CSAllocatable::alloc_from_witness(cs, Some(payload.root))?;
        Ok(Self {
            magic,
            payload_type,
            slot,
            ring_size,
            root,
        })
    }
}

#[cfg(test)]
mod tests {
    use pairing::bn256::Bn256;
    use sync_vm::franklin_crypto::bellman::SynthesisError;

    use crate::utils::{
        bytes_constant_from_hex_str,
        testing::{bytes_assert_eq, create_test_constraint_system},
    };

    #[test]
    fn test_wormhole_payload() -> Result<(), SynthesisError> {
        let hex_str = "415557560000000000069b993c00002710095bb7e5fa374ea08603a6698123d99101547a50";
        let bytes = bytes_constant_from_hex_str::<Bn256>(hex_str)?;
        let payload = super::WormholePayload::new_from_slice(&bytes)?;
        {
            bytes_assert_eq(&payload.magic, "41555756");
            bytes_assert_eq(&payload.payload_type, "00");
            bytes_assert_eq(&payload.slot, "00000000069b993c");
            bytes_assert_eq(&payload.ring_size, "00002710");
            bytes_assert_eq(&payload.root, "095bb7e5fa374ea08603a6698123d99101547a50");
        }

        bytes_assert_eq(&payload.to_bytes(), hex_str);
        Ok(())
    }

    #[test]
    fn test_wormhole_body() -> Result<(), SynthesisError> {
        let hex_str = "655ccff800000000001ae101faedac5851e32b9b23b5f9411a8c2bac4aae3ed4dd7b811dd1a72ea4aa71000000000195faa401415557560000000000069b993c00002710095bb7e5fa374ea08603a6698123d99101547a50";
        let bytes = bytes_constant_from_hex_str::<Bn256>(hex_str)?;
        let body = super::WormholeBody::new_from_slice(&bytes)?;
        {
            bytes_assert_eq(&body.timestamp, "655ccff8");
            bytes_assert_eq(&body.nonce, "00000000");
            bytes_assert_eq(&body.emitter_chain, "001a");
            bytes_assert_eq(
                &body.emitter_address,
                "e101faedac5851e32b9b23b5f9411a8c2bac4aae3ed4dd7b811dd1a72ea4aa71",
            );
            bytes_assert_eq(&body.sequence, "000000000195faa4");
            bytes_assert_eq(&body.consistency_level, "01");
            bytes_assert_eq(
                &body.payload.to_bytes(),
                "415557560000000000069b993c00002710095bb7e5fa374ea08603a6698123d99101547a50",
            );
        }

        bytes_assert_eq(&body.to_bytes(), hex_str);
        Ok(())
    }

    #[test]
    fn test_payload_alloc_from_witness() -> Result<(), SynthesisError> {
        let cs = &mut create_test_constraint_system()?;
        let hex_str = "415557560000000000069b993c00002710095bb7e5fa374ea08603a6698123d99101547a50";
        let data = hex::decode(hex_str).unwrap();
        let payload = pythnet_sdk::wire::v1::WormholeMessage::try_from_bytes(&data).unwrap();
        let payload = super::WormholePayload::<_>::alloc_from_witness(cs, payload)?;
        bytes_assert_eq(&payload.to_bytes(), hex_str);
        Ok(())
    }

    #[test]
    fn test_body_alloc_from_witness() -> Result<(), SynthesisError> {
        let cs = &mut create_test_constraint_system()?;
        let data = hex::decode(get_vaa()).unwrap();
        let vaa: wormhole_sdk::Vaa<&serde_wormhole::RawMessage> =
            serde_wormhole::from_slice(&data).unwrap();
        let (_, body): (_, wormhole_sdk::vaa::Body<_>) = vaa.into();
        let expected = hex::encode(serde_wormhole::to_vec(&body).unwrap());
        let body = super::WormholeBody::<_>::alloc_from_witness(cs, body)?;
        bytes_assert_eq(&body.to_bytes(), expected);
        Ok(())
    }

    #[test]
    fn test_vaa_alloc_from_witness() -> Result<(), SynthesisError> {
        let cs = &mut create_test_constraint_system()?;
        let data = hex::decode(get_vaa()).unwrap();
        let vaa: wormhole_sdk::Vaa<&serde_wormhole::RawMessage> =
            serde_wormhole::from_slice(&data).unwrap();
        let _ = super::WormholeMessage::<_>::alloc_from_witness(cs, vaa)?;
         
        // let (_, body): (_, wormhole_sdk::vaa::Body<_>) = vaa.into();
        // let expected = hex::encode(serde_wormhole::to_vec(&body).unwrap());
        // let body = super::WormholeBody::<_>::alloc_from_witness(cs, body)?;
        // bytes_assert_eq(&body.to_bytes(), expected);
        Ok(())
    }

    fn get_vaa() -> &'static str {
        "01000000030d00d5df1d274a402c5eb4c8b60254f1d1df67c64c6afddd75ed03562aac6d4ad0714bd0874f0837683bec3357999a4c2d922f79e908c39a5a6ff4ec6e21a78956fa00021e32f66495cb657049f04b251629811395d082d4aecee8a95e447e83372a4e9443a647f44880f3da72d58dfc0f9fa963e4aac0c283342d9a91c4e19d3ca62a5b0103381bfdf0853bbf0f7b4cb4d65851ac7f60dcc9ba3d8442c95de61410cbf09ef279454fa725fd2e90697f55e065005ad64e6696c009fd1767b7bf9b79738399bf00068260c97865c386a3496aa56da2327159998ab1db26ae79010685f75518d4eecb67cda0cda4408a636301d0d376f3ff71db66f088e24d871bf8f9d75f901b84e8010743b8b7f7b4d53e5499bc0d2548a952cb2b6559da1a0583d3128d930926c6cf281ff58828c54cc9e39c774b70fb5ab7ab400eaa6356bc06700b2f744c6a13fd06010859f92b8bd6fa6cb257d5a41327b48c2ac880773eda6617f8511a8003a56fff15502b2b90f65cbe16ddfda2324e3d0b4039fba3332cde2adf48f01e46e8717839000a2fcf534a53c3e53addf02dea50a6e87b20f41922708a38768af6ad48dc53ca0f65844530c842f2746ecef4a950843e2adfdd1f8765e3a172e346a793fe136b90010bf3022b0f4927b6b701a84e949da4cfacbc8cc2e72037516c1ba12ef7a354e77c454822878d7d948e50c0e7118cfca2a4d5a33810e7c5cf63a47a0115cb3c5f98000c06c01308e45e4d95711e735ef2ef9e5eddeaf1e0a52faf28e0e9cb2b37acde794557d6ce463ac7b9c16f753ddd142f5716c64bfe3c9c01960f07d46cafd7157e010d5cd199cddb07c62c95eb3d199a324e79392562af5568a33842e23c1a0f2550a1010f6a4af293d651e13acb8a5f1967da722df8422ee871731ca0d9e0a908fc7f010ecc18446ff3bf2a129401967556df7de3bbfcc2c37d4441cde11d71b86a8128aa22e2154e4943570aed1d2aaa747ddc10729702688b70751a9d9c411b9e0271da0010922dd9890ea99eb32ffb3fe2fcda2258b875147601af4bad528edf70a33f382b79b4ef1515a7c5aa60af16a75c555d714b4ce7b31275d4b4eb427089849ff0920012997ca65ec7fcf0418fd036ddead5743206a7a350fd44602759a4bba2acfc949924244db3d12d76885c162b988135e642c1d6c27aa4ba504668c7932d37ead91b00655ccff800000000001ae101faedac5851e32b9b23b5f9411a8c2bac4aae3ed4dd7b811dd1a72ea4aa71000000000195faa401415557560000000000069b993c00002710095bb7e5fa374ea08603a6698123d99101547a50"
    }
}