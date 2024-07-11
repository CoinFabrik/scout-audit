use anyhow::{Context, Result};
use cargo_metadata::Metadata;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Copy, Clone, EnumIter, Display, EnumString)]
pub enum BlockChain {
    Ink,
    Soroban,
}

impl BlockChain {
    pub fn variants() -> Vec<String> {
        Self::iter().map(|e| e.to_string()).collect()
    }

    pub fn get_detectors_url(&self) -> &'static str {
        match self {
            BlockChain::Ink => "https://github.com/CoinFabrik/scout",
            BlockChain::Soroban => "https://github.com/CoinFabrik/scout-soroban",
        }
    }

    #[tracing::instrument(name = "GET BLOCKCHAIN DEPENDENCY", level = "debug", skip_all)]
    pub fn get_blockchain_dependency(metadata: &Metadata) -> Result<Self> {
        let blockchain = metadata
            .packages
            .iter()
            .find_map(|p| match p.name.as_str() {
                "soroban-sdk" => Some(BlockChain::Soroban),
                "ink" => Some(BlockChain::Ink),
                _ => None,
            })
            .with_context(|| {
                let supported_blockchains = BlockChain::variants().join(", ");
                format!(
                    "Could not find any supported blockchain dependency in the Cargo.toml file.\n   Supported blockchains include:\n   - {}\n",
                    supported_blockchains.replace(", ", "\n   - ")
                )
            })?;
        Ok(blockchain)
    }
}
