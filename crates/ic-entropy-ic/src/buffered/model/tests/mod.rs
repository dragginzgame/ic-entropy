use super::*;

fn insert(state: &mut Model, seed: [u8; 32], now: u64) {
    let claim = state.begin(now).unwrap();
    assert!(state.begin(now).is_none());
    state.finish(claim, Ok(seed), now);
}

#[test]
fn opt_out_stays_idle_and_full_pool_stops_until_draw() {
    let mut state = Model::new();
    assert_eq!(state.status().refill, RefillState::Disabled);
    assert!(state.begin(0).is_none());
    assert_eq!(state.take_seed(), Err(Error::Disabled));
    state.reset(true, 0);
    assert_eq!(state.take_seed(), Err(Error::NotReady));
    for index in 0..POOL_CAPACITY {
        insert(
            &mut state,
            [u8::try_from(index).unwrap(); 32],
            index as u64 * SECOND,
        );
        assert!(state.begin(index as u64 * SECOND).is_none());
    }
    assert_eq!(state.status().refill, RefillState::Idle);
    assert!(state.begin(u64::MAX).is_none());
    assert_eq!(state.take_seed(), Ok([0; 32]));
    assert_eq!(state.status().refill, RefillState::Scheduled);
}

#[test]
fn refill_backoff_and_failure_budget_require_explicit_retry() {
    let mut state = Model::new();
    state.reset(true, 0);
    let mut now = 0;
    for failure in 1..=MAX_FAILURES {
        let claim = state.begin(now).unwrap();
        state.finish(claim, Err(Error::Transport), now);
        assert_eq!(state.status().consecutive_failures, failure);
        let next = now + (1_u64 << (failure - 1)).min(60) * SECOND;
        assert_eq!(state.status().next_refill_ns, next);
        assert!(state.begin(next - 1).is_none());
        now = next;
    }
    assert_eq!(state.status().refill, RefillState::Paused);
    assert_eq!(state.take_seed(), Err(Error::RetryLimit));
    assert!(state.begin(u64::MAX).is_none());
    state.retry(now).unwrap();
    insert(&mut state, [4; 32], now);
    assert_eq!(state.status().consecutive_failures, 0);
    assert_eq!(state.take_seed(), Ok([4; 32]));
}

#[test]
fn bounded_draws_refuse_without_advancing_or_changing_output() {
    let mut state = Model::new();
    state.reset(true, 0);
    insert(&mut state, [7; 32], 0);
    let mut expected = Generator::from_seed([7; 32], BYTE_BUDGET);
    let mut large = [99; MAX_DRAW_BYTES + 1];
    assert_eq!(state.fill_bytes(&mut large), Err(Error::DrawTooLarge));
    assert_eq!(large, [99; MAX_DRAW_BYTES + 1]);
    assert_eq!(state.status().buffered_seeds, 1);
    let mut actual = [0; MAX_DRAW_BYTES];
    let mut reference = [0; MAX_DRAW_BYTES];
    for _ in 0..BYTE_BUDGET / MAX_DRAW_BYTES as u64 {
        state.fill_bytes(&mut actual).unwrap();
        expected.fill_bytes(&mut reference).unwrap();
        assert_eq!(actual, reference);
    }
    assert_eq!(state.status().remaining_bytes, 0);
    actual.fill(99);
    assert_eq!(state.fill_bytes(&mut actual), Err(Error::Exhausted));
    assert_eq!(actual, [99; MAX_DRAW_BYTES]);
    assert_eq!(state.status().remaining_bytes, 0);
    state.fill_bytes(&mut []).unwrap();
}

#[test]
fn replacement_and_retry_do_not_replay_existing_stream() {
    let mut state = Model::new();
    state.reset(true, 0);
    insert(&mut state, [1; 32], 0);
    let mut bytes = [0; MAX_DRAW_BYTES];
    for _ in 0..BYTE_BUDGET / MAX_DRAW_BYTES as u64 {
        state.fill_bytes(&mut bytes).unwrap();
    }
    insert(&mut state, [2; 32], SECOND);
    state.retry(SECOND).unwrap();
    state.fill_bytes(&mut bytes).unwrap();
    let mut expected = [0; MAX_DRAW_BYTES];
    Generator::from_seed([2; 32], BYTE_BUDGET)
        .fill_bytes(&mut expected)
        .unwrap();
    assert_eq!(bytes, expected);
    assert_eq!(
        state.status().remaining_bytes,
        BYTE_BUDGET - MAX_DRAW_BYTES as u64
    );
}

#[test]
fn reconstruction_discards_stream_and_fences_old_success_failure_and_cleanup() {
    let mut state = Model::new();
    state.reset(true, 0);
    insert(&mut state, [1; 32], 0);
    state.fill_bytes(&mut [0; 16]).unwrap();
    let old = state.begin(SECOND).unwrap();
    state.reset(false, 0);
    state.finish(old, Ok([2; 32]), SECOND);
    assert_eq!(state.status().buffered_seeds, 0);
    assert_eq!(state.status().remaining_bytes, 0);
    state.reset(true, SECOND);
    let replacement = state.begin(SECOND).unwrap();
    state.finish(old, Err(Error::Transport), SECOND);
    assert_eq!(state.status().refill, RefillState::Pending);
    assert_eq!(state.status().consecutive_failures, 0);
    assert_eq!(state.fill_bytes(&mut [0; 16]), Err(Error::NotReady));
    state.finish(replacement, Ok([3; 32]), SECOND);
    assert_eq!(state.take_seed(), Ok([3; 32]));
    assert_eq!(state.take_seed(), Err(Error::Exhausted));
}

#[test]
fn source_refusals_and_cancellation_preserve_available_generator() {
    let mut state = Model::new();
    state.reset(true, 0);
    insert(&mut state, [9; 32], 0);
    state.fill_bytes(&mut [0; 16]).unwrap();
    let remaining = state.status().remaining_bytes;
    for (index, error) in [
        Error::Busy,
        Error::MalformedReply,
        Error::Stale,
        Error::Transport,
    ]
    .into_iter()
    .enumerate()
    {
        let now = state.status().next_refill_ns;
        let claim = state.begin(now).unwrap();
        assert_eq!(state.retry(now), Err(Error::Busy));
        state.finish(claim, Err(error), now);
        assert_eq!(state.status().last_error, Some(error));
        assert_eq!(usize::from(state.status().consecutive_failures), index + 1);
        assert_eq!(state.status().remaining_bytes, remaining);
    }
    state.retry(state.status().next_refill_ns).unwrap();
    assert_eq!(state.status().remaining_bytes, remaining);
}
