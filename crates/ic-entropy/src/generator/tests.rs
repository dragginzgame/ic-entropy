use super::*;

#[test]
fn preserves_icydb_word_order_and_successive_draws() {
    let seed = [17; 32];
    let mut original = ChaCha20Rng::from_seed(seed);
    let mut generator = Generator::from_seed(seed, 64);
    for _ in 0..4 {
        let expected = (u128::from(original.next_u64()) << 64) | u128::from(original.next_u64());
        assert_eq!(generator.next_u128(), Ok(expected));
    }
    assert_eq!(generator.next_u128(), Err(Exhausted));
}

#[test]
fn rejected_draw_preserves_output_budget_and_stream() {
    let seed = [42; 32];
    let mut generator = Generator::from_seed(seed, 16);
    let mut output = [0xab; 17];
    assert_eq!(generator.fill_bytes(&mut output), Err(Exhausted));
    assert_eq!(output, [0xab; 17]);
    assert_eq!(generator.remaining_bytes(), 16);
    assert_eq!(
        generator.next_u128(),
        Generator::from_seed(seed, 16).next_u128()
    );
    assert_eq!(generator.remaining_bytes(), 0);
    assert_eq!(generator.fill_bytes(&mut []), Ok(()));
}

#[test]
fn reseeding_replaces_stream_and_restores_budget() {
    let mut generator = Generator::from_seed([1; 32], 16);
    generator.next_u128().unwrap();
    generator.reseed([2; 32], 32);
    let mut expected = Generator::from_seed([2; 32], 32);
    let mut actual = [0; 32];
    let mut wanted = [0; 32];
    generator.fill_bytes(&mut actual).unwrap();
    expected.fill_bytes(&mut wanted).unwrap();
    assert_eq!(actual, wanted);
    assert_eq!(generator.remaining_bytes(), 0);
}

#[test]
fn zero_budget_refuses_nonempty_draws() {
    let mut generator = Generator::from_seed([0; 32], 0);
    assert_eq!(generator.next_u128(), Err(Exhausted));
    assert_eq!(generator.fill_bytes(&mut [0; 1]), Err(Exhausted));
}

#[test]
fn pooled_consumers_keep_independent_budgets_and_streams() {
    let mut pool = crate::SeedPool::<2>::new();
    pool.insert([1; 32]).unwrap();
    pool.insert([2; 32]).unwrap();
    let mut short = Generator::from_seed(pool.take().unwrap(), 2_000);
    let mut long = Generator::from_seed(pool.take().unwrap(), 20_000);
    assert_eq!(pool.take(), None);

    short.fill_bytes(&mut [0; 2_000]).unwrap();
    let mut refused = [0xab; 8];
    assert_eq!(short.fill_bytes(&mut refused), Err(Exhausted));
    assert_eq!(refused, [0xab; 8]);
    assert_eq!(long.remaining_bytes(), 20_000);

    let mut expected = Generator::from_seed([2; 32], 20_000);
    assert_eq!(long.next_u128(), expected.next_u128());
    pool.insert([3; 32]).unwrap();
    short.reseed(pool.take().unwrap(), 2_000);
    assert_eq!(long.next_u128(), expected.next_u128());
    assert_eq!(long.remaining_bytes(), 19_968);
    assert_eq!(short.remaining_bytes(), 2_000);
}
