//! Test fixture assembled from genuine accepted transitions and independent
//! upstream serialization. Earlier model transitions are not on-chain evidence.
use anchor_lang::solana_program::system_program;
use piv1::integrations::FeeFraction;
use super::{bootstrap_custody::{Fixture, POOL, RESERVE, MANAGER},
    custody::{CONFIG, PRINCIPAL_SOL, PRINCIPAL_JITO, MINT},
    support::{kif_claim_custody::BackingAccount, vault_custody_model::World}};

pub fn principal(sol: u64, tokens: u64, shared: bool) -> Fixture {
    let mut world = World::empty(3, 42);
    if sol != 0 { world.explicit_sol(sol, sol).unwrap(); }
    if tokens != 0 { world.explicit_tokens(tokens, tokens).unwrap(); }
    world.bootstrap().unwrap(); from_world(&world, shared)
}
pub fn from_world(world: &World, shared: bool) -> Fixture {
    let mut f = Fixture::from_world(world, shared); let pool = world.pool.raw_snapshot();
    f.pool.total_lamports = pool.total_pool_lamports; f.pool.pool_token_supply = pool.pool_token_supply;
    f.pool.last_update_epoch = pool.last_update_epoch; f.clock.epoch = pool.current_epoch;
    f.mint.supply = pool.pool_token_supply;
    f.pool.sol_deposit_fee = super::oracle::Fee { numerator: 0, denominator: 0 };
    f.pool.sol_deposit_authority = None;
    for account in &mut f.custody.accounts { account.writable = false; }
    let referrer = if shared { MANAGER } else { 18 };
    for index in [CONFIG, PRINCIPAL_SOL, PRINCIPAL_JITO, MINT, POOL, RESERVE, MANAGER, referrer] {
        f.custody.accounts[index].writable = true;
    }
    let withdraw = f.mint.mint_authority.unwrap();
    f.custody.accounts.push(BackingAccount { key: withdraw, owner: system_program::ID, lamports: 0,
        data: vec![], signer: false, writable: false, executable: false });
    refresh(&mut f); f
}
pub fn refresh(f: &mut Fixture) {
    f.sync();
    // Pool Borsh serialization writes only the logical prefix. Keep a substantial
    // nonzero allocation tail in every ordinary success and failure oracle.
    f.custody.accounts[POOL].data.resize(2048, 0xd3);
    f.custody.accounts[POOL].lamports = f.custody.rent.minimum_balance(2048);
}
pub fn completed_world() -> World {
    let mut w = World::new(10_000, 50, 111, 9, 3, FeeFraction::ZERO, 100_000);
    w.round.bump = w.config.bumps.active_distribution;
    w.open(900_000).unwrap(); w.settle().unwrap(); w.integrate(900_100).unwrap();
    w.explicit_sol(23, 23).unwrap(); w.explicit_tokens(11, 11).unwrap(); w
}
