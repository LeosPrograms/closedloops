use core::fmt::Debug;

pub trait AccountId: Clone + Ord + Debug {}

impl AccountId for i32 {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Node<Id: AccountId> {
    Source,
    Sink,
    WithId(Id),
}

impl<Id> From<Id> for Node<Id>
where
    Id: AccountId,
{
    fn from(id: Id) -> Self {
        Self::WithId(id)
    }
}
