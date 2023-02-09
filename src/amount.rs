use core::fmt::{Debug, Display};
use core::iter::Sum;
use core::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use num_traits::{One, Zero};

pub trait Amount:
    Copy
    + Sum<Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Neg<Output = Self>
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

impl Amount for i32 {}
