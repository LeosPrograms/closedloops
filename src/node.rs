use crate::Id;

/// A node type used to model a balanced obligation network.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Node<N> {
    Source,
    Sink,
    WithId(N),
}

impl<N> From<N> for Node<N>
where
    N: Id,
{
    fn from(id: N) -> Self {
        Self::WithId(id)
    }
}
