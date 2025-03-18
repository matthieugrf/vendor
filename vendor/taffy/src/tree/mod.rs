//! Contains both a high-level interface to Taffy using a ready-made node tree, and a set of traits for defining custom node trees.
//!
//! - For documentation on the high-level API, see the [`TaffyTree`] struct.
//! - For documentation on the low-level trait-based API, see the [`traits`] module.

// Submodules
mod cache;
mod layout;
mod node;
pub mod traits;

pub use cache::Cache;
pub use layout::{CollapsibleMarginSet, Layout, LayoutInput, LayoutOutput, RequestedAxis, RunMode, SizingMode};
pub use node::NodeId;
pub(crate) use traits::LayoutPartialTreeExt;
pub use traits::{LayoutPartialTree, PrintTree, RoundTree, TraversePartialTree, TraverseTree};

#[cfg(feature = "taffy_tree")]
mod taffy_tree;
#[cfg(feature = "taffy_tree")]
pub use taffy_tree::{TaffyError, TaffyResult, TaffyTree};
