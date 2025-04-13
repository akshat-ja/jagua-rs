use std::time::Instant;
use slotmap::SecondaryMap;
use crate::entities::bin_packing::problem::LayKey;
use crate::entities::general::layout::LayoutSnapshot;
use crate::fsize;

#[cfg(doc)]
use crate::entities::bin_packing::problem::BPProblem;

/// Snapshot of [`BPProblem`] at a specific moment.
/// Can be used to restore [`BPProblem`] to a previous state.
#[derive(Debug, Clone)]
pub struct BPSolution {
    /// Snapshots of all `Layout`s in the `Problem` at the moment the solution was created
    pub layout_snapshots: SecondaryMap<LayKey, LayoutSnapshot>,
    /// Average usage of bins in the solution
    pub usage: fsize,
    /// Quantity of placed items for each `Item` in the solution
    pub placed_item_qtys: Vec<usize>,
    /// Target quantity of each `Item` in the solution
    pub target_item_qtys: Vec<usize>,
    /// Quantity of bins used for each type of bin
    pub bin_qtys: Vec<usize>,
    /// Instant the solution was created
    pub time_stamp: Instant,
}