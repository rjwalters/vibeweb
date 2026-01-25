//! DOM types, traversal, and mutation APIs
//!
//! This crate provides the core DOM (Document Object Model) types used by
//! the vibeweb browser. It uses an arena-based allocation strategy where
//! nodes are stored in a vector and referenced by indices, avoiding the
//! complexity of Rc<RefCell<Node>> patterns.

mod node;
mod document;
mod traverse;
mod debug;

pub use node::{Node, NodeData, NodeId, ElementData};
pub use document::Document;
pub use traverse::{ChildIter, DescendantIter, Ancestors};

#[cfg(test)]
mod tests;
