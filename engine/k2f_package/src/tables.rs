use crate::error::PackageError;
use crate::package::Package;
use k2f_core::{
    collapse_tables_to_assets, expand_manifest_tables_with_assets, table_asset_sources, AssetsMap,
};
use std::collections::BTreeMap;

impl Package {
    /// Expand asset-backed tables to inline rows. Returns id → asset path for later collapse.
    pub fn expand_table_assets(&mut self) -> Result<BTreeMap<String, String>, PackageError> {
        let sources = table_asset_sources(&self.engine_manifest());
        if sources.is_empty() {
            return Ok(sources);
        }
        let assets: AssetsMap = self.assets.clone().into_iter().collect();
        let expanded = expand_manifest_tables_with_assets(&self.engine_manifest(), &assets)
            .map_err(PackageError::Other)?;
        self.root = expanded.root;
        self.manifest.running_blocks = expanded.running_blocks;
        Ok(sources)
    }

    pub fn collapse_table_assets(
        &mut self,
        sources: &BTreeMap<String, String>,
    ) -> Result<(), PackageError> {
        if sources.is_empty() {
            return Ok(());
        }
        let mut assets: AssetsMap = self.assets.clone().into_iter().collect();
        collapse_tables_to_assets(&mut self.root, sources, &mut assets)
            .map_err(PackageError::Other)?;
        for rb in &mut self.manifest.running_blocks {
            collapse_tables_to_assets(&mut rb.node, sources, &mut assets)
                .map_err(PackageError::Other)?;
        }
        self.assets = assets.into_iter().collect();
        Ok(())
    }
}
