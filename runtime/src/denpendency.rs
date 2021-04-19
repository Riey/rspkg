use crate::{BuildArtifacts, Error, Result};
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use rspkg_shared::DependencyType;
use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering::SeqCst},
};

#[derive(Debug, Default, Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub ty: DependencyType,
    pub no_default_features: bool,
    pub features: HashSet<String>,
}

impl DependencyInfo {
    pub fn merge(&mut self, other: &Self) {
        debug_assert_eq!(self.name, other.name);
        debug_assert_eq!(self.ty, other.ty);

        self.no_default_features |= other.no_default_features;
        self.features.extend(other.features.iter().cloned());
    }
}

#[derive(Default)]
pub struct DependencyStore {
    next: AtomicU32,
    dependency_names: DashMap<(String, DependencyType), u32>,
    dependencies: DashMap<u32, DependencyInfo>,
    artifacts: DashMap<u32, BuildArtifacts>,
}

impl DependencyStore {
    fn next_index(&self) -> u32 {
        self.next.fetch_add(1, SeqCst)
    }

    pub fn get_artifact(&self, index: u32) -> Result<Ref<u32, BuildArtifacts>> {
        self.artifacts
            .get(&index)
            .ok_or_else(|| Error::ArtifactNotFound(index))
    }

    pub fn add_artifact(&self, artifact: BuildArtifacts) -> u32 {
        let next = self.next_index();

        assert!(self.artifacts.insert(next, artifact).is_none());

        next
    }

    pub fn add_dependency(&self, dep: DependencyInfo) -> u32 {
        let index = *self
            .dependency_names
            .entry((dep.name.clone(), dep.ty))
            .or_insert(self.next_index())
            .value();

        let entry = self.dependencies.entry(index);

        entry.and_modify(|d| d.merge(&dep)).or_insert(dep);

        index
    }

    pub fn add_feature(&self, index: u32, feature: impl Into<String>) -> Result<()> {
        let mut dep = self.get_mut_dependency(index)?;
        dep.features.insert(feature.into());
        Ok(())
    }

    pub fn get_mut_dependency(&self, index: u32) -> Result<RefMut<u32, DependencyInfo>> {
        self.dependencies
            .get_mut(&index)
            .ok_or_else(|| Error::ArtifactNotFound(index))
    }

    pub fn get_dependency(&self, index: u32) -> Result<Ref<u32, DependencyInfo>> {
        self.dependencies
            .get(&index)
            .ok_or_else(|| Error::ArtifactNotFound(index))
    }
}
