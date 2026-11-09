use super::*;

#[test]
fn bounded_fifo_wraps_without_overwriting_or_replaying() {
    let mut pool = SeedPool::<2>::new();
    assert_eq!(pool.take(), None);
    pool.insert([1; 32]).unwrap();
    pool.insert([2; 32]).unwrap();
    assert_eq!(pool.insert([3; 32]), Err(PoolFull));
    assert_eq!(pool.take(), Some([1; 32]));
    pool.insert([3; 32]).unwrap();
    assert_eq!(pool.take(), Some([2; 32]));
    assert_eq!(pool.take(), Some([3; 32]));
    assert_eq!(pool.take(), None);
    assert!(pool.is_empty());
}

#[test]
fn zero_capacity_refuses_without_panicking() {
    let mut pool = SeedPool::<0>::new();
    assert!(pool.is_full());
    assert_eq!(pool.insert([1; 32]), Err(PoolFull));
    assert_eq!(pool.take(), None);
}
