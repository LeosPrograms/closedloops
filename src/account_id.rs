use core::fmt::Debug;

pub trait AccountIdTrait: Clone + Ord + Debug {}

impl AccountIdTrait for i32 {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Node<Id: AccountIdTrait> {
    Source,
    Sink,
    WithId(Id),
}

impl<Id> From<Id> for Node<Id>
where
    Id: AccountIdTrait,
{
    fn from(id: Id) -> Self {
        Self::WithId(id)
    }
}
