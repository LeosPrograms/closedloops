use core::fmt::Display;

pub trait AccountIdTrait: Clone + Ord + Display {
    fn source() -> Self;
    fn sink() -> Self;
}

impl AccountIdTrait for i32 {
    fn source() -> Self {
        Self::MAX
    }

    fn sink() -> Self {
        Self::MIN
    }
}
