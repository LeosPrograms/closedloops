use crate::AccountId;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum Node<Id> {
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
