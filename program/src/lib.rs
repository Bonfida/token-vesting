

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;


pub mod instruction;
pub mod error;
pub mod state;

pub mod processor;
