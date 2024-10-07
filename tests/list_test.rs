use rand::distributions::Alphanumeric;
use rand::Rng;
use libtheia::crdt::{ List, CmRDT };

type SiteId = u32;

#[test]
fn test_new() {
    let site1: List<char, SiteId> = List::new();
    assert_eq!(site1.len(), 0);
    assert!(site1.is_empty());
}

#[test]
fn test_is_empty() {
    let mut site1 = List::new();
    assert!(site1.is_empty());

    let o = site1.insert_index(0, 'a', 'A');
    site1.apply(o);
    assert!(!site1.is_empty());
}

#[test]
fn test_append() {
    let mut site1 = List::new();
    assert!(site1.is_empty());

    let op = site1.append('a', 0);
    site1.apply(op);
    let op = site1.append('b', 0);
    site1.apply(op);
    let op = site1.append('c', 0);
    site1.apply(op);

    assert_eq!(String::from_iter(site1), "abc");
}

#[test]
fn test_out_of_order_inserts() {
    let mut site1 = List::new();
    let mut site2 = List::new();
    let op1 = site1.insert_index(0, 'a', 0);
    site1.apply(op1.clone());

    let op2 = site1.insert_index(1, 'c', 0);
    site1.apply(op2.clone());

    let op3 = site1.insert_index(1, 'b', 0);
    site1.apply(op3.clone());

    let mut ops = vec![op1, op2, op3];
    let mut iterations = 0;
    while let Some(op) = ops.pop() {
        assert!(iterations < (3 * (3 + 1)) / 2);
        iterations += 1;
        if site2.validate_operation(&op).is_ok() {
            site2.apply(op)
        } else {
            ops.insert(0, op);
        }
    }

    let site1_items = String::from_iter(site1);
    assert_eq!(site1_items, "abc");
    assert_eq!(site1_items, String::from_iter(site2));
}

#[test]
fn test_concurrent_inserts_with_same_identifier_can_be_split() {
    let mut list_a = List::new();
    let mut list_b = List::new();
    let mut list_c = List::new();

    let op_a = list_a.insert_index(0, 'a', 'A');
    let op_b = list_b.insert_index(0, 'b', 'B');

    list_a.apply(op_a.clone());
    list_a.apply(op_b.clone());
    list_b.apply(op_a.clone());
    list_b.apply(op_b.clone());
    list_c.apply(op_a);
    list_c.apply(op_b);

    assert_eq!(list_a.read::<String>(), "ab");
    assert_eq!(list_b.read::<String>(), "ab");
    assert_eq!(list_c.read::<String>(), "ab");
    list_c.apply(list_c.insert_index(1, 'c', 'C'));
    assert_eq!(list_c.read::<String>(), "acb");
}

#[test]
fn test_append_mixed_with_inserts() {
    let mut site1 = List::new();
    let op = site1.append('a', 0);
    site1.apply(op);

    let op = site1.insert_index(0, 'b', 0);
    site1.apply(op);

    let op = site1.append('c', 0);
    site1.apply(op);

    let op = site1.insert_index(1, 'd', 0);
    site1.apply(op);

    assert_eq!(String::from_iter(site1), "bdac");
}

#[test]
fn test_delete_of_index() {
    let mut site1 = List::new();
    let op = site1.insert_index(0, 'a', 0);
    site1.apply(op);
    let op = site1.insert_index(1, 'b', 0);
    site1.apply(op);
    assert_eq!(String::from_iter(site1.iter()), "ab");

    let op = site1.delete_index(0, 0);
    site1.apply(op.unwrap());
    assert_eq!(String::from_iter(site1), "b");
}

#[test]
fn test_position() {
    let mut site1 = List::new();
    let op = site1.append('a', 0);
    site1.apply(op);
    let op = site1.append('b', 0);
    site1.apply(op);

    assert_eq!(site1.pos(0), Some(&'a'));
    assert_eq!(site1.pos(1), Some(&'b'));
}

#[test]
fn test_identifier_position() {
    let mut site1 = List::new();
    let op_a = site1.append('a', 0);
    site1.apply(op_a.clone());
    let op_b = site1.append('b', 0);
    site1.apply(op_b.clone());
    let op_c = site1.append('c', 0);

    assert_eq!(site1.pos_entry(op_a.id()), Some(0));
    assert_eq!(site1.pos_entry(op_b.id()), Some(1));
    assert_eq!(site1.pos_entry(op_c.id()), None);
}

#[test]
fn test_reapply_list_ops() {
    let mut rng = rand::thread_rng();

    let s1 = rng.clone().sample_iter(Alphanumeric).map(char::from);

    let mut site1 = List::new();
    let mut site2 = List::new();

    for c in s1.take(5000) {
        let ix = rng.gen_range(0..site1.len() + 1);
        let insert_op = site1.insert_index(ix, c, 0);
        site1.apply(insert_op.clone());

        site2.apply(insert_op.clone());
        site2.apply(insert_op.clone());

        let delete_op = site2.delete_index(ix, 1).unwrap();
        // apply op a coupel of times
        site2.apply(delete_op.clone());
        site2.apply(delete_op.clone());
        // apply op a coupel of times
        site1.apply(delete_op.clone());
        site1.apply(delete_op);

        // now try applying insert op again (even though delete already appled)
        site1.apply(insert_op.clone());
    }

    assert!(
        site1.is_empty(),
        "site1 was not empty: {}",
        String::from_iter(site1)
    );
    assert!(
        site2.is_empty(),
        "site2 was not empty: {}",
        String::from_iter(site2)
    );

    assert_eq!(String::from_iter(site2), String::from_iter(site1));
}

#[test]
fn test_insert_followed_by_deletes() {
    let mut rng = rand::thread_rng();

    let s1 = rng.clone().sample_iter(Alphanumeric).map(char::from);

    let mut site1 = List::new();
    let mut site2 = List::new();

    for c in s1.take(5000) {
        let ix = rng.gen_range(0..site1.len() + 1);
        let insert_op = site1.insert_index(ix, c, 0);
        site1.apply(insert_op.clone());
        site2.apply(insert_op);

        let delete_op = site2.delete_index(ix, 1).unwrap();
        site2.apply(delete_op.clone());
        site1.apply(delete_op);
    }

    assert!(
        site1.is_empty(),
        "site1 was not empty: {}",
        String::from_iter(site1)
    );
    assert!(
        site2.is_empty(),
        "site2 was not empty: {}",
        String::from_iter(site2)
    );
}

#[test]
fn test_mutual_insert_qc1() {
    let mut site0 = List::new();
    let mut site1 = List::new();
    let plan = vec![
        (8, 24, false),
        (23, 1, true),
        (93, 94, false),
        (68, 30, false),
        (37, 27, true),
    ];

    for (elem, idx, source_is_site0) in plan {
        let ((source, source_actor), replica) = if source_is_site0 {
            ((&mut site0, 0), &mut site1)
        } else {
            ((&mut site1, 1), &mut site0)
        };
        let i = idx % (source.len() + 1);
        println!("{:?} inserting {} @ {}", source_actor, elem, i);
        let op = source.insert_index(i, elem, source_actor);
        source.apply(op.clone());
        replica.apply(op);
    }

    assert_eq!(Vec::from_iter(site0), Vec::from_iter(site1));
}

#[test]
fn test_deep_inserts() {
    let mut site = List::new();

    let mut vec = Vec::new();
    let n = 1000;
    for v in 0..n {
        let i = site.len() / 2;
        println!("inserting {}/{}", i, site.len());
        vec.insert(i, v);
        let op = site.insert_index(i, v, 0);
        site.apply(op);
    }
    assert_eq!(site.len(), n);
    assert_eq!(Vec::from_iter(site), vec);
}
