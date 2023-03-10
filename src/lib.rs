//! Rust implementations of algorithms and tools for Multilateral Trade Credit Set-off (MTCS).
//!
//! This crate provides -
//! * A library containing implementations of algorithms used for MTCS.
//! * A CLI tool that runs MTCS on a specified input CSV file (containing a list of obligations) and
//! outputs the resulting set-off notices as a CSV file.
//!
//! This crate implements MTCS using the minimum-cost maximum-flow algorithms based on ideas from
//! the following paper -
//! [Fleischman, T.; Dini, P. Mathematical Foundations for Balancing the Payment System in the Trade Credit Market. J. Risk Financial Manag. 2021, 14, 452](https://doi.org/10.3390/jrfm14090452)
//!

#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod algo;
pub mod error;
pub mod id;
pub mod impls;
pub mod int;
pub mod node;
pub mod obligation;
pub mod setoff;

pub use impls::complex_id::ComplexIdMtcs;
pub use impls::complex_id_map::ComplexIdMapMtcs;
pub use impls::default::DefaultMtcs;

use crate::obligation::Obligation;
use crate::setoff::SetOff;

pub trait Mtcs {
    type Obligation: Obligation;
    type SetOff: SetOff;
    type Obligations: IntoIterator<Item = Self::Obligation>;
    type SetOffs: IntoIterator<Item = Self::SetOff>;
    type Algo;
    type Error;

    fn run(&mut self, obligations: Self::Obligations) -> Result<Self::SetOffs, Self::Error>;
    fn check(&self, setoffs: &Self::SetOffs) -> Result<(), Self::Error>;
}
