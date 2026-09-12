pub fn percentile_nearest_rank(samples: &mut [u128], percentile: u32) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    samples.sort_unstable();
    let rank = ((samples.len() as u128 * percentile as u128 + 99) / 100) as usize;
    samples[rank.saturating_sub(1)]
}
