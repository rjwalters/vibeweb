//! DOM node types and structures

/// A unique identifier for a node within a Document.
///
/// NodeId is an index into the Document's node arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    /// Returns the raw index value.
    pub fn index(self) -> usize {
        self.0
    }
}

/// Data specific to element nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    /// The element's tag name (e.g., "div", "p", "html").
    pub tag_name: String,
    /// The element's attributes as key-value pairs.
    pub attributes: Vec<(String, String)>,
}

impl ElementData {
    /// Creates a new ElementData with the given tag name and no attributes.
    pub fn new(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes: Vec::new(),
        }
    }

    /// Creates a new ElementData with the given tag name and attributes.
    pub fn with_attributes(tag_name: impl Into<String>, attributes: Vec<(String, String)>) -> Self {
        Self {
            tag_name: tag_name.into(),
            attributes,
        }
    }

    /// Gets the value of an attribute by name.
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Sets an attribute value, replacing any existing value.
    pub fn set_attribute(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();
        if let Some((_, v)) = self.attributes.iter_mut().find(|(k, _)| k == &name) {
            *v = value;
        } else {
            self.attributes.push((name, value));
        }
    }
}

/// The type-specific data stored in a DOM node.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    /// The root document node.
    Document,
    /// An element node (e.g., <div>, <p>).
    Element(ElementData),
    /// A text node containing character data.
    Text(String),
    /// A comment node.
    Comment(String),
    /// A doctype node.
    Doctype {
        /// The doctype name (usually "html").
        name: String,
    },
}

/// A DOM node in the document tree.
///
/// Nodes form a tree structure through parent/child/sibling relationships.
/// All relationships are stored as optional NodeIds to support the arena-based
/// allocation strategy.
#[derive(Debug, Clone)]
pub struct Node {
    /// The type-specific data for this node.
    pub data: NodeData,
    /// Parent node, if any.
    pub parent: Option<NodeId>,
    /// First child node, if any.
    pub first_child: Option<NodeId>,
    /// Last child node, if any.
    pub last_child: Option<NodeId>,
    /// Previous sibling node, if any.
    pub prev_sibling: Option<NodeId>,
    /// Next sibling node, if any.
    pub next_sibling: Option<NodeId>,
}

impl Node {
    /// Creates a new node with the given data and no relationships.
    pub fn new(data: NodeData) -> Self {
        Self {
            data,
            parent: None,
            first_child: None,
            last_child: None,
            prev_sibling: None,
            next_sibling: None,
        }
    }

    /// Returns true if this node is an element node.
    pub fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    /// Returns true if this node is a text node.
    pub fn is_text(&self) -> bool {
        matches!(self.data, NodeData::Text(_))
    }

    /// Returns true if this node is a document node.
    pub fn is_document(&self) -> bool {
        matches!(self.data, NodeData::Document)
    }

    /// Returns the element data if this is an element node.
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns mutable element data if this is an element node.
    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the text content if this is a text node.
    pub fn as_text(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text(text) => Some(text),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId(42);
        assert_eq!(id.index(), 42);
    }

    #[test]
    fn test_element_data() {
        let mut elem = ElementData::new("div");
        assert_eq!(elem.tag_name, "div");
        assert!(elem.attributes.is_empty());

        elem.set_attribute("class", "container");
        assert_eq!(elem.get_attribute("class"), Some("container"));

        elem.set_attribute("class", "wrapper");
        assert_eq!(elem.get_attribute("class"), Some("wrapper"));
        assert_eq!(elem.attributes.len(), 1);
    }

    #[test]
    fn test_node_type_checks() {
        let doc = Node::new(NodeData::Document);
        assert!(doc.is_document());
        assert!(!doc.is_element());
        assert!(!doc.is_text());

        let elem = Node::new(NodeData::Element(ElementData::new("p")));
        assert!(elem.is_element());
        assert!(!elem.is_document());
        assert!(!elem.is_text());

        let text = Node::new(NodeData::Text("Hello".to_string()));
        assert!(text.is_text());
        assert!(!text.is_document());
        assert!(!text.is_element());
    }
}
