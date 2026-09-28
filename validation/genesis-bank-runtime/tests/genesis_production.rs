//! Real production initializer ELF; shared independent Bank custody/state oracles.
#[path = "genesis_bank.rs"]
mod harness;

#[test]
fn production_initializer_four_profiles() { harness::run_profiles(true); }
