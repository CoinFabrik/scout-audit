use std::path::PathBuf;

use anyhow::Result;
use cargo::Config;
use itertools::Itertools;

use crate::scout::blockchain::BlockChain;

use self::{
    builder::DetectorBuilder,
    configuration::{DetectorConfiguration, DetectorsConfigurationList},
};
mod builder;
mod configuration;
mod library;
mod source;

use cargo_metadata::Metadata;
pub use configuration::{get_local_detectors_configuration, get_remote_detectors_configuration};

#[derive(Debug)]
pub struct Detectors<'a> {
    cargo_config: Config,
    detectors_configs: DetectorsConfigurationList,
    metadata: &'a Metadata,
    verbose: bool,
}

impl<'a> Detectors<'a> {
    /// Creates a new instance of `Detectors`
    pub fn new(
        cargo_config: Config,
        detectors_configs: DetectorsConfigurationList,
        metadata: &'a Metadata,
        verbose: bool,
    ) -> Self {
        Self {
            cargo_config,
            detectors_configs,
            metadata,
            verbose,
        }
    }

    /// Builds detectors and returns the paths to the built libraries
    pub fn build(self, bc: BlockChain, used_detectors: &[String]) -> Result<Vec<PathBuf>> {
        self.detectors_configs
            .iter()
            .map(|detectors_config| self.build_detectors(detectors_config, bc, used_detectors))
            .flatten_ok()
            .collect::<Result<Vec<_>>>()
    }

    pub fn get_detector_names(&self) -> Result<Vec<String>> {
        self.detectors_configs
            .iter()
            .map(|detectors_config| {
                let builder = DetectorBuilder::new(
                    &self.cargo_config,
                    detectors_config,
                    self.metadata,
                    self.verbose,
                );
                builder.get_detector_names()
            })
            .flatten_ok()
            .collect::<Result<Vec<_>>>()
    }

    fn build_detectors(
        &self,
        detectors_config: &DetectorConfiguration,
        bc: BlockChain,
        used_detectors: &[String],
    ) -> Result<Vec<PathBuf>> {
        let builder = DetectorBuilder::new(
            &self.cargo_config,
            detectors_config,
            self.metadata,
            self.verbose,
        );
        builder.build(bc, used_detectors.to_vec())
    }
}
