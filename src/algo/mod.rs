pub mod mcmf;

pub trait FlowPath {
    type Node;
    type Flow;
    type Iter: IntoIterator<Item = Self::Node>;

    fn nodes(&self) -> Self::Iter;
    fn flow(&self) -> Self::Flow;
}
