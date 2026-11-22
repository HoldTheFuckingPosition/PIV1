//! Offline Borsh witness from the retained candidate host library. No runtime or wallet.
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
use piv1::{
    accounts::{CONFIG_DISCRIMINATOR, DISTRIBUTION_DISCRIMINATOR},
    state::{ActiveDistribution, PivConfig},
};
use std::{env, fs, path::Path};

fn write<T: AnchorSerialize>(directory: &Path, name: &str, value: &T, discriminator: [u8; 8], size: usize) {
    let mut bytes = discriminator.to_vec();
    value.serialize(&mut bytes).unwrap();
    assert!(bytes.len() <= size);
    bytes.resize(size, 0);
    fs::write(directory.join(name), bytes).unwrap();
}

fn main() {
    let mut args = env::args().skip(1);
    let previous = fs::read(args.next().unwrap()).unwrap();
    let destination = args.next().unwrap();
    let directory = Path::new(&destination);
    assert!(args.next().is_none());
    assert_eq!(&previous[..8], &CONFIG_DISCRIMINATOR);
    let mut config = PivConfig::deserialize(&mut &previous[8..]).unwrap();
    assert_eq!(config.version, 1);
    config.validate_initialized().unwrap();
    config.record_initial_storage_reserve(650_240);
    config.paused = false;
    // This synthetic state tests independent fields and u64 safety, including
    // a value beyond JavaScript's exact Number range. It is not a migration.
    for (index, value) in [650_240u64, 701_123, 742_777, 807_111, u64::MAX].into_iter().enumerate() {
        config.migration_reserve[index * 8..index * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    for (name, first, second) in [
        ("config-v2-absent.bin", None, None),
        ("config-v2-one.bin", Some(101), None),
        ("config-v2-two.bin", Some(101), Some(202)),
    ] {
        config.last_successful_preparation_at = first;
        config.last_valid_insufficient_attempt_at = second;
        config.validate_initialized().unwrap();
        write(directory, name, &config, CONFIG_DISCRIMINATOR, PivConfig::SPACE);
    }
    let mut round = ActiveDistribution::new_idle(config.bumps.active_distribution);
    write(directory, "round-v1.bin", &round, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SPACE);
    round.version = ActiveDistribution::SWAP_VERSION;
    write(directory, "round-v2.bin", &round, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::SWAP_SPACE);
    let recovery = round.migrate_v2_recovery().unwrap();
    write(directory, "round-v3.bin", &recovery, DISTRIBUTION_DISCRIMINATOR, ActiveDistribution::RECOVERY_SPACE);
}
