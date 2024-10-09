use libtheia::crdt::{CmRDT, CvRDT, Map, Version};
use libtheia::crdt::map::Operation as MapOperation;
use libtheia::crdt::multi_value::Operation as MultiValueOperation;
use libtheia::crdt::multi_value::MultiValue;

#[test]
fn test_new() {
    let m: Map<u8, MultiValue<u8, _>, u8> = Map::new();
    assert_eq!(m.len().value, 0);
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

    let a = m.get(&1).derive_add(1);
    let o = m.update(1, a, |map, x| {
        map.update(2, x, |mv, x| mv.write(2, x))
    });

    assert_eq!(
        o,
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

    m.apply(o);

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

    let a = m.len().derive_add(1);
    let mut im: Map<u8, MultiValue<u8, u8>, u8> = Map::new();
    im.apply(im.update(1, a, |mv, x| mv.write(0, x)));

    let add_a = m.len().derive_add(1);
    m.apply(m.update(2, add_a, |map, x| {
        map.update(1, x, |mv, x| mv.write(0, x))
    }));

    assert_eq!(m.get(&2).value, Some(im));
    assert_eq!(m.len().value, 1);

    m.apply(m.remove(2, m.get(&2).derive_remove()));

    assert_eq!(m.get(&2).value, None);
    assert_eq!(m.len().value, 0);
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

    let m1_c = m1.clone();

    m1.merge(m2.clone());
    m2.merge(m1_c);
    assert_eq!(m1, m2);

    let inner_map = m1.get(&1).value.unwrap();
    assert_eq!(inner_map.get(&3).value.map(|r| r.read().value), Some(vec![4]));
    assert_eq!(inner_map.get(&2).value, None);
    assert_eq!(inner_map.len().value, 1);
}
