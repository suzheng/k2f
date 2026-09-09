use crate::error::{map_compile_message, AgentError};
use k2f_layout::compile_manifest;
use k2f_package::{apply_coverage_subset, Package};
use std::collections::HashMap;

pub(crate) fn write_lock(pkg: &mut Package) -> Result<(), AgentError> {
    apply_coverage_subset(&mut pkg.fonts).map_err(AgentError::from)?;
    let assets: HashMap<String, Vec<u8>> = pkg.assets.clone().into_iter().collect();
    let lock = compile_manifest(
        pkg.engine_manifest(),
        &pkg.theme_json,
        &pkg.fonts,
        if assets.is_empty() {
            None
        } else {
            Some(&assets)
        },
    )
    .map_err(|e| map_compile_message(&e))?;
    pkg.set_lock(&lock)?;
    Ok(())
}

pub(crate) fn relock(pkg: &mut Package) -> Result<(), AgentError> {
    pkg.signatures_json = None;
    write_lock(pkg)
}
