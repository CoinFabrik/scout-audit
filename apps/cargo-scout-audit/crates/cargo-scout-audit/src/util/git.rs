use anyhow::{Result, anyhow, bail, ensure};
use cargo::{
    GlobalContext,
    core::{Dependency, Package, PackageId},
    sources::source::{MaybePackage, QueryKind, Source},
    util::cache_lock::CacheLockMode,
};
use std::path::PathBuf;

/// Downloads git repo using cargo native cache and returns its path.
pub fn download_git_repo(dependency: &Dependency, config: &GlobalContext) -> Result<PathBuf> {
    let _lock = config.acquire_package_cache_lock(CacheLockMode::DownloadExclusive)?;
    let mut source = dependency.source_id().load(config, &Default::default())?;
    let package_id = sample_package_id(dependency, &mut *source)?;

    if let MaybePackage::Ready(package) = source.download(package_id)? {
        git_dependency_root_from_package(config, &*source, &package)
    } else {
        bail!(format!("`{}` is not ready", package_id.name()))
    }
}

fn sample_package_id(dep: &Dependency, source: &mut dyn Source) -> anyhow::Result<PackageId> {
    let mut package_id: Option<PackageId> = None;

    while {
        let poll = source.query(dep, QueryKind::Alternatives, &mut |summary| {
            if package_id.is_none() {
                package_id = Some(summary.package_id());
            }
        })?;
        if poll.is_pending() {
            source.block_until_ready()?;
            package_id.is_none()
        } else {
            false
        }
    } {}

    package_id.ok_or_else(|| anyhow!("Found no packages in `{}`", dep.source_id()))
}

fn git_dependency_root_from_package<'a>(
    config: &'a GlobalContext,
    source: &(dyn Source + 'a),
    package: &Package,
) -> anyhow::Result<PathBuf> {
    let package_root = package.root();

    if source.source_id().is_git() {
        let git_path = config.git_path();
        let git_path =
            config.assert_package_cache_locked(CacheLockMode::DownloadExclusive, &git_path);
        ensure!(
            package_root.starts_with(git_path.join("checkouts")),
            "Unexpected path: {}",
            package_root.to_string_lossy()
        );
        let n = git_path.components().count() + 3;
        Ok(package_root.components().take(n).collect())
    } else if source.source_id().is_path() {
        unreachable!()
    } else {
        unimplemented!()
    }
}
