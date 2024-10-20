use core::fmt;

pub enum NodeError {
    TooManyChildren,
    TooManyNodes,
    UnknownID(u32),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyChildren => write!(f, "the node has too many children"),
            Self::TooManyNodes => write!(f, "the tree has too many nodes"),
            Self::UnknownID(id) => write!(f, "there is no node with id {id}"),
        }
    }
}
