pub fn percentile_nearest_rank(samples: &mut [u128], percentile: u32) -> u128 {
    percentile_nearest_rank_thousandths(samples, percentile * 1_000)
}

pub fn percentile_nearest_rank_thousandths(
    samples: &mut [u128],
    percentile_thousandths: u32,
) -> u128 {
    assert!(!samples.is_empty());
    assert!((1_000..=100_000).contains(&percentile_thousandths));
    samples.sort_unstable();
    let rank =
        ((samples.len() as u128 * percentile_thousandths as u128 + 99_999) / 100_000) as usize;
    samples[rank.saturating_sub(1)]
}
