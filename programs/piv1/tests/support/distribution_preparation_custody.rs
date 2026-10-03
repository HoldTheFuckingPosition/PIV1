//! Independently encoded current registry/rewards/Clock added to fixed custody.
use anchor_lang::{prelude::{Pubkey, SolanaSysvar}, AnchorDeserialize,
    solana_program::sysvar};
use piv1::{accounts::DISTRIBUTION_DISCRIMINATOR,
    guardian_clock_accounts::GUARDIAN_REGISTRY_DISCRIMINATOR,
    kif_claim_accounts::GUARDIAN_REWARD_DISCRIMINATOR,
    state::{ActiveDistribution, GuardianRegistry, GuardianReward}};
use super::{bootstrap_custody, support::{kif_claim_custody::{BackingAccount, envelope, key, PROGRAM},
    vault_custody_model::World}, custody::*};
#[derive(Clone, Debug, PartialEq)]
pub struct Fixture { pub base: bootstrap_custody::Fixture, pub registry: GuardianRegistry,
    pub rewards: [GuardianReward; 6], pub guardian_start: usize }
impl Fixture {
    pub fn new(pending: u64, carry: u64, shared: bool, bitmap: u8) -> Self {
        let w = World::new(pending, 73, carry, 19, 3, piv1::integrations::FeeFraction::ZERO, 100_000);
        let mut base = bootstrap_custody::Fixture::from_world(&w, shared);
        let (registry_key, bump) = Pubkey::find_program_address(&[b"guardian-registry"], &PROGRAM);
        base.custody.edit_config(|c| { c.guardian_registry = registry_key; c.bumps.guardian_registry = bump;
            c.kif_anchor_timestamp = 0; });
        let registry = GuardianRegistry::new(bump, base.custody.config().guardian_registry_revision,
            core::array::from_fn(|i| key(100 + i as u8))).unwrap();
        let state = |key, data: Vec<u8>| BackingAccount { key, owner: PROGRAM, executable: false,
            signer: false, writable: false, lamports: base.custody.rent.minimum_balance(data.len()), data };
        let mut extras = vec![state(registry_key, envelope(&registry, GUARDIAN_REGISTRY_DISCRIMINATOR, GuardianRegistry::SPACE))];
        let rewards = core::array::from_fn(|i| {
            let mut r = GuardianReward::new(0, &registry, i as u8).unwrap();
            r.last_active_period = if bitmap & (1 << i) != 0 { Some(0) } else { None };
            r.claimable_lamports = 10 + i as u64; r.cumulative_earned = r.claimable_lamports;
            let (key, bump) = Pubkey::find_program_address(&[b"guardian-reward", r.guardian.as_ref(),
                &r.registry_revision.to_le_bytes(), &[i as u8]], &PROGRAM);
            r.bump = bump; extras.push(state(key, envelope(&r, GUARDIAN_REWARD_DISCRIMINATOR, GuardianReward::SPACE))); r
        });
        extras.push(BackingAccount { key: sysvar::clock::ID, owner: sysvar::ID, signer: false,
            writable: false, executable: false, lamports: 0, data: vec![0;40] });
        let guardian_start = base.custody.accounts.len(); base.custody.accounts.extend(extras);
        for i in [CONFIG, ROUND, PENDING_SOL, PRINCIPAL_SOL, ESCROW_SOL] { base.custody.accounts[i].writable = true; }
        let mut f = Self { base, registry, rewards, guardian_start }; f.sync_clock(); f
    }
    pub fn round(&self) -> ActiveDistribution {
        ActiveDistribution::deserialize(&mut &self.base.custody.accounts[ROUND].data[8..]).unwrap()
    }
    pub fn set_round(&mut self, r: ActiveDistribution) {
        self.base.custody.accounts[ROUND].data = envelope(&r, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE);
    }
    pub fn sync_clock(&mut self) {
        self.base.clock.to_account_info(&mut self.base.custody.accounts[self.guardian_start+7].info()).unwrap();
    }
}
