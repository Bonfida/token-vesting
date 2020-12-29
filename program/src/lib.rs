

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;


pub mod instruction;
mod error;
mod state;

pub mod processor;


