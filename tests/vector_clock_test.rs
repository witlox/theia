use std::fmt::Debug;
use libtheia::crdt::{Reset, Version, CvRDT, CmRDT, VectorClock};

#[test]
fn test_reset_remove() {
    let mut a: VectorClock<u8> = vec![Version::new(1, 4), Version::new(2, 3), Version::new(5, 9)]
        .into_iter()
        .collect();
    let b: VectorClock<u8> = vec![Version::new(1, 5), Version::new(2, 3), Version::new(5, 8)]
        .into_iter()
        .collect();
    let expected: VectorClock<u8> = vec![Version::new(5, 9)].into_iter().collect();

    a.reset(&b);
    assert_eq!(a, expected);
}

#[test]
fn test_merge() {
    let mut a: VectorClock<u8> = vec![Version::new(1, 1), Version::new(4, 4)].into_iter().collect();
    let b: VectorClock<u8> = vec![Version::new(3, 3), Version::new(4, 3)].into_iter().collect();

    a.merge(b);

    let expected: VectorClock<u8> = vec![Version::new(1, 1), Version::new(3, 3), Version::new(4, 4)]
        .into_iter()
        .collect();

    assert_eq!(a, expected);
}

#[test]
fn test_merge_less_left() {
    let (mut a, mut b) = (VectorClock::new(), VectorClock::new());
    a.apply(Version::new(5, 5));
    b.apply(Version::new(6, 6));
    b.apply(Version::new(7, 7));

    a.merge(b);
    assert_eq!(a.get(&5), 5);
    assert_eq!(a.get(&6), 6);
    assert_eq!(a.get(&7), 7);
}

#[test]
fn test_merge_less_right() {
    let (mut a, mut b) = (VectorClock::new(), VectorClock::new());
    a.apply(Version::new(6, 6));
    a.apply(Version::new(7, 7));
    b.apply(Version::new(5, 5));

    a.merge(b);
    assert_eq!(a.get(&5), 5);
    assert_eq!(a.get(&6), 6);
    assert_eq!(a.get(&7), 7);
}

#[test]
fn test_merge_same_id() {
    let (mut a, mut b) = (VectorClock::new(), VectorClock::new());
    a.apply(Version::new(1, 1));
    a.apply(Version::new(2, 1));
    b.apply(Version::new(1, 1));
    b.apply(Version::new(3, 1));

    a.merge(b);
    assert_eq!(a.get(&1), 1);
    assert_eq!(a.get(&2), 1);
    assert_eq!(a.get(&3), 1);
}

#[test]
#[allow(clippy::neg_cmp_op_on_partial_ord)]
fn test_vector_clock_ordering() {
    assert_eq!(VectorClock::<i8>::new(), VectorClock::new());

    let (mut a, mut b) = (VectorClock::new(), VectorClock::new());
    a.apply(Version::new("A".to_string(), 1));
    a.apply(Version::new("A".to_string(), 2));
    a.apply(Version::new("A".to_string(), 0));
    b.apply(Version::new("A".to_string(), 1));

    assert!(a > b);
    assert!(b < a);
    assert_ne!(a, b);

    b.apply(Version::new("A".to_string(), 3));
    assert!(b > a);
    assert!(a < b);
    assert_ne!(a, b);

    a.apply(Version::new("B".to_string(), 1));
    assert_ne!(a, b);
    assert!(!(a > b));
    assert!(!(b > a));

    a.apply(Version::new("A".to_string(), 3));
    assert!(a > b);
    assert!(b < a);
    assert_ne!(a, b);

    b.apply(Version::new("B".to_string(), 2));
    assert!(b > a);
    assert!(a < b);
    assert_ne!(a, b);

    a.apply(Version::new("B".to_string(), 2));
    assert!(!(b > a));
    assert!(!(a > b));
    assert_eq!(a, b);
}
