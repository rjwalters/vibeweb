//! DOM types, traversal, and mutation APIs
//!
//! This crate provides the core DOM (Document Object Model) types used by
//! the vibeweb browser. It uses an arena-based allocation strategy where
//! nodes are stored in a vector and referenced by indices, avoiding the
//! complexity of Rc<RefCell<Node>> patterns.

mod document;
mod node;
mod traverse;
mod debug;

pub use document::Document;
pub use node::{ElementData, Node, NodeData, NodeId};
pub use traverse::{Ancestors, ChildIter, DescendantIter};

#[cfg(test)]
mod tests;
