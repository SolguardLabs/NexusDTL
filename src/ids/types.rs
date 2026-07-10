use serde::{Serialize, Serializer};

use crate::{NexusResult, canonical_bytes};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AccountId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VaultId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FacilityId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IntentId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BatchId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TxId([u8; 32]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Digest([u8; 32]);

macro_rules! id_type {
    ($name:ident) => {
        impl $name {
            pub const fn from_bytes(bytes: [u8; 32]) -> Self {
                Self(bytes)
            }

            pub const fn bytes(self) -> [u8; 32] {
                self.0
            }

            pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> NexusResult<Self> {
                let digest = Digest::from_serializable(domain, value)?;
                Ok(Self(digest.bytes()))
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&hex::encode(self.0))
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "{}", hex::encode(self.0))
            }
        }
    };
}

id_type!(AccountId);
id_type!(AssetId);
id_type!(VaultId);
id_type!(FacilityId);
id_type!(IntentId);
id_type!(BatchId);
id_type!(TxId);

impl Digest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn from_parts(domain: &str, parts: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        for part in parts {
            hasher.update(&(part.len() as u64).to_be_bytes());
            hasher.update(part);
        }
        Self(*hasher.finalize().as_bytes())
    }

    pub fn from_serializable<T: Serialize>(domain: &str, value: &T) -> NexusResult<Self> {
        let bytes = canonical_bytes(value)?;
        Ok(Self::from_parts(domain, &[&bytes]))
    }
}

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", hex::encode(self.0))
    }
}

impl AssetId {
    pub fn derive(symbol: &str, decimals: u8) -> Self {
        Digest::from_parts("nexus-asset-v1", &[symbol.as_bytes(), &[decimals]]).into()
    }
}

impl VaultId {
    pub fn derive(controller: AccountId, reserve_asset: AssetId, salt: Digest) -> Self {
        Digest::from_parts(
            "nexus-vault-v1",
            &[&controller.bytes(), &reserve_asset.bytes(), &salt.bytes()],
        )
        .into()
    }
}

impl FacilityId {
    pub fn derive(vault_id: VaultId, borrower: AccountId, collateral_asset: AssetId) -> Self {
        Digest::from_parts(
            "nexus-facility-v1",
            &[
                &vault_id.bytes(),
                &borrower.bytes(),
                &collateral_asset.bytes(),
            ],
        )
        .into()
    }
}

impl From<Digest> for AccountId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for AssetId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for VaultId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for FacilityId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for IntentId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for BatchId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}

impl From<Digest> for TxId {
    fn from(value: Digest) -> Self {
        Self(value.bytes())
    }
}
