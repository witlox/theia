use libtheia::CRDT::{ CmRDT, CvRDT, Map, Version};
use libtheia::CRDT::map::Operation as MapOperation;
use libtheia::CRDT::multi_value::Operation as MultiValueOperation;
use libtheia::CRDT::multi_value::MultiValue;

#[test]
fn test_new() {
    let m: Map<u8, MultiValue<u8, _>, u8> = Map::new();
    assert_eq!(m.length().value, 0);
    assert!(m.is_empty().value);
}

#[test]
fn test_is_empty() {
    let mut m: Map<&str, Map<&str, MultiValue<&str, _>, _>, &str> = Map::new();
    let is_empty_read = m.is_empty();
    assert!(is_empty_read.value);

    m.apply(
        m.update("a", is_empty_read.derive_add("A"), |map, x| {
            map.update("b", x, |mv, x| mv.write("b", x))
        }),
    );

    assert!(!m.is_empty().value);
}

#[test]
fn test_update() {
    let mut m: Map<u8, Map<u8, MultiValue<u8, u8>, u8>, u8> = Map::new();

    let add_a = m.get(&1).derive_add(1);
    let op_a = m.update(1, add_a, |map, x| {
        map.update(2, x, |mv, x| mv.write(2, x))
    });

    assert_eq!(
        op_a,
        MapOperation::Update {
            version: Version::new(1, 1),
            key: 1,
            operation: MapOperation::Update {
                version: Version::new(1, 1),
                key: 2,
                operation: MultiValueOperation::Put {
                    clock: Version::new(1, 1).into(),
                    value: 2
                }
            }
        }
    );

    assert_eq!(m, Map::new());

    m.apply(op_a);

    assert_eq!(
        m.get(&1)
            .value
            .and_then(|m2| m2.get(&2).value)
            .map(|r| r.read().value),
        Some(vec![2])
    );

    m.apply(m.update(1, m.get(&1).derive_add(1), |map, x| {
        map.update(2, x, |mv, x| {
            assert_eq!(mv.read().value, vec![2]);
            mv.write(3, x)
        })
    }));

    assert_eq!(
        m.get(&1)
            .value
            .and_then(|m2| m2.get(&2).value)
            .map(|r| r.read().value),
        Some(vec![3])
    );
}

#[test]
fn test_remove() {
    let mut m: Map<u8, Map<u8, MultiValue<u8, u8>, u8>, u8> = Map::new();

    let add_a = m.length().derive_add(1);
    let mut inner_map: Map<u8, MultiValue<u8, u8>, u8> = Map::new();
    inner_map.apply(inner_map.update(1, add_a, |mv, x| mv.write(0, x)));

    let add_a = m.length().derive_add(1);
    m.apply(m.update(2, add_a, |map, x| {
        map.update(1, x, |mv, x| mv.write(0, x))
    }));

    assert_eq!(m.get(&2).value, Some(inner_map));
    assert_eq!(m.length().value, 1);

    m.apply(m.remove(2, m.get(&2).derive_remove()));

    assert_eq!(m.get(&2).value, None);
    assert_eq!(m.length().value, 0);
}

#[test]
fn test_reset_remove_semantics() {
    let mut m1: Map<u8, Map<u8, MultiValue<u8, u8>, u8>, u8> = Map::new();

    m1.apply(m1.update(1, m1.get(&1).derive_add(1), |map, x| {
        map.update(2, x, |mv, x| mv.write(2, x))
    }));

    let mut m2 = m1.clone();

    m1.apply(m1.remove(1, m1.get(&1).derive_remove()));

    m2.apply(m2.update(1, m2.get(&1).derive_add(3), |map, x| {
        map.update(3, x, |mv, x| mv.write(4, x))
    }));

    let m1_snapshot = m1.clone();

    m1.merge(m2.clone());
    m2.merge(m1_snapshot);
    assert_eq!(m1, m2);

    let inner_map = m1.get(&1).value.unwrap();
    assert_eq!(inner_map.get(&3).value.map(|r| r.read().value), Some(vec![4]));
    assert_eq!(inner_map.get(&2).value, None);
    assert_eq!(inner_map.length().value, 1);
}
