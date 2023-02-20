use std::ops::Index;

use crate::flatten::structures::{
    index_trait::{IndexRange, IndexRef},
    indexed_map::{AuxillaryMap, IndexedMap},
};

use super::{control::structures::ControlIdx, prelude::*};

/// A structure which contains the basic information about a component
/// definition needed during simulation.
#[derive(Debug)]
pub struct ComponentCore {
    /// The control program for this component.
    pub control: Option<ControlIdx>,
    /// The set of assignments that are always active.
    pub continuous_assignments: IndexRange<AssignmentIdx>,
    /// True iff component is combinational
    pub is_comb: bool,
}

pub struct AuxillaryComponentInfo {
    /// Name of the component.
    pub name: Identifier,

    /// The input/output signature of this component.
    pub inputs: IndexRange<LocalPortRef>,
    pub outputs: IndexRange<LocalPortRef>,

    pub names: PortNames,
    pub cell_info: CellInfoMap,
}

#[derive(Debug)]
pub struct PortNames {
    pub port_names: AuxillaryMap<LocalPortRef, Identifier>,
    pub ref_port_names: AuxillaryMap<LocalRPortRef, Identifier>,
}

impl PortNames {
    /// Creates a new [`CompNames`] struct with the default value for the
    /// auxillary maps being the empty string.
    pub fn new() -> Self {
        let default = Identifier::get_default_id();
        Self {
            port_names: AuxillaryMap::new_with_default(default),
            ref_port_names: AuxillaryMap::new_with_default(default),
        }
    }
}

impl Default for PortNames {
    fn default() -> Self {
        Self::new()
    }
}

pub type ComponentMap = IndexedMap<ComponentRef, ComponentCore>;

// NOTHING IMPORTANT DOWN HERE, DO NOT READ
// =======================================

/// IGNORE FOR NOW
///
///  A map from various local references to the name of the port/cell
///
/// The basic idea is to have a single vector of the names densely packed and to
/// have the separate types be distinct regions of the vector.
pub struct CompactLocalNameMap {
    port_base: usize,
    cell_base: usize,
    rport_base: usize,
    rcell_base: usize,
    names: Vec<Identifier>,
}

impl CompactLocalNameMap {
    /// Creates a new [`CompactLocalNameMap`] with the given capacity.
    pub fn with_capacity(size: usize) -> Self {
        Self {
            port_base: usize::MAX,
            cell_base: usize::MAX,
            rport_base: usize::MAX,
            rcell_base: usize::MAX,
            names: Vec::with_capacity(size),
        }
    }
    /// Creates a new [`CompactLocalNameMap`].
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
}

impl Default for CompactLocalNameMap {
    fn default() -> Self {
        Self::new()
    }
}

// Lots index trait implementations, not interesting I promise

impl Index<PortRef> for CompactLocalNameMap {
    type Output = Identifier;

    fn index(&self, index: PortRef) -> &Self::Output {
        match index {
            PortRef::Local(idx) => {
                debug_assert!(self.port_base != usize::MAX);
                &self.names[self.port_base + idx.index()]
            }
            PortRef::Ref(idx) => {
                debug_assert!(self.rport_base != usize::MAX);
                &self.names[self.rport_base + idx.index()]
            }
        }
    }
}

impl Index<LocalPortRef> for CompactLocalNameMap {
    type Output = Identifier;

    fn index(&self, index: LocalPortRef) -> &Self::Output {
        debug_assert!(self.port_base != usize::MAX);
        &self.names[self.port_base + index.index()]
    }
}

impl Index<LocalRPortRef> for CompactLocalNameMap {
    type Output = Identifier;

    fn index(&self, index: LocalRPortRef) -> &Self::Output {
        debug_assert!(self.rport_base != usize::MAX);
        &self.names[self.rport_base + index.index()]
    }
}

impl Index<LocalRCellRef> for CompactLocalNameMap {
    type Output = Identifier;

    fn index(&self, index: LocalRCellRef) -> &Self::Output {
        debug_assert!(self.rcell_base != usize::MAX);
        &self.names[self.rcell_base + index.index()]
    }
}

impl Index<LocalCellRef> for CompactLocalNameMap {
    type Output = Identifier;

    fn index(&self, index: LocalCellRef) -> &Self::Output {
        debug_assert!(self.cell_base != usize::MAX);
        &self.names[self.cell_base + index.index()]
    }
}
