//! External protocol boundary. No CPI is implemented in Task 1.1.

pub mod jito;
pub mod stake_pool;

pub use jito::JitoStrategy;
pub use stake_pool::StakePoolAdapter;
