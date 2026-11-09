use super::*;

#[test]
fn dropped_refill_reopens_demand_but_cannot_clear_replacement_instance() {
    BUFFER.with_borrow_mut(|state| state.reset(true, 10));
    let generation = BUFFER.with_borrow_mut(|state| state.begin(10)).unwrap();
    drop(RefillRequest {
        generation,
        started_ns: 10,
        completed: false,
    });
    assert_eq!(status().consecutive_failures, 1);
    assert_eq!(status().refill, RefillState::Scheduled);
    let next = status().next_refill_ns;
    let generation = BUFFER.with_borrow_mut(|state| state.begin(next)).unwrap();
    let old = RefillRequest {
        generation,
        started_ns: next,
        completed: false,
    };
    BUFFER.with_borrow_mut(|state| state.reset(true, next));
    assert!(BUFFER.with_borrow_mut(|state| state.begin(next)).is_some());
    drop(old);
    assert_eq!(status().refill, RefillState::Pending);
    assert_eq!(status().consecutive_failures, 0);
}
