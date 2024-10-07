use num::BigRational;
use libtheia::crdt::Identifier;

#[test]
fn test_adding_zero_node_makes_identifier_smaller() {
    let id_a = Identifier(vec![
        (BigRational::new(0.into(), 1.into()), 0),
        (BigRational::new(0.into(), 1.into()), 0),
    ]);
    let id_b = Identifier(vec![(BigRational::new(0.into(), 1.into()), 0)]);
    assert!(id_a < id_b);
}

#[test]
fn test_id_is_dense_qc1() {
    let id_a = Identifier(vec![
        (BigRational::new(0i64.into(), 1i64.into()), 0),
        (BigRational::new(0i64.into(), 1.into()), 0),
    ]);
    let id_b = Identifier(vec![(BigRational::new(0i64.into(), 1i64.into()), 0)]);
    println!("id_a: {}", id_a);
    println!("id_b: {}", id_b);
    println!("id_a < id_b: {:?}", id_a < id_b);
    println!("id_b < id_a: {:?}", id_b < id_a);
    assert!(id_a < id_b);

    let id_mid = Identifier::between(Some(&id_a), Some(&id_b), 0);
    println!("minmax: {}, {}", id_a, id_b);
    assert!(id_a < id_mid, "{} < {}", id_a, id_mid);
    assert!(id_mid < id_b, "{} < {}", id_mid, id_b);
}

#[test]
fn test_id_is_dense_qc2() {
    let id_a = Identifier(vec![
        (BigRational::new(0.into(), 1.into()), 1),
        (BigRational::new((-1).into(), 1.into()), 0),
    ]);
    let id_b = Identifier(vec![
        (BigRational::new(0.into(), 1.into()), 0),
        (BigRational::new(0.into(), 1.into()), 0),
    ]);
    let marker = 0;

    let (id_min, id_max) = if id_a < id_b {
        (id_a, id_b)
    } else {
        (id_b, id_a)
    };

    let id_mid = Identifier::between(Some(&id_min), Some(&id_max), marker);

    if id_min == id_max {
        assert_eq!(id_min, id_mid);
        assert_eq!(id_max, id_mid);
    } else {
        assert!(id_min < id_mid, "{} < {}", id_min, id_mid);
        assert!(id_mid < id_max, "{} < {}", id_mid, id_max);
    }
}

#[test]
fn test_id_is_dense_qc3() {
    let (id_a, id_b, marker) = (
        Identifier(vec![(BigRational::new(0.into(), 1.into()), 1)]),
        Identifier(vec![(BigRational::new(0.into(), 1.into()), 0)]),
        0,
    );
    let (id_min, id_max) = if id_a < id_b {
        (id_a, id_b)
    } else {
        (id_b, id_a)
    };

    let id_mid = Identifier::between(Some(&id_min), Some(&id_max), marker);

    if id_min == id_max {
        assert_eq!(id_min, id_mid);
        assert_eq!(id_max, id_mid);
    } else {
        assert!(id_min < id_mid, "{} < {}", id_min, id_mid);
        assert!(id_mid < id_max, "{} < {}", id_mid, id_max);
    }
}
