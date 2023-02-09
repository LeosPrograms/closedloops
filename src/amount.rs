use core::fmt::{Debug, Display};
use core::iter::Sum;
use core::ops::{Add, AddAssign, Sub, SubAssign};
use num_traits::{One, Zero};

pub trait AmountTrait:
    Copy
    + Sum<Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Ord
    + AddAssign
    + SubAssign
    + Zero
    + One
    + Debug
    + Display
    + Default
{
}

impl AmountTrait for i32 {}
