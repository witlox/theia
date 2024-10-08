use rand::distributions::Alphanumeric;
use rand::Rng;
use libtheia::crdt::{ List, CmRDT };

type SiteId = u32;

#[test]
fn test_new() {
    let lst: List<char, SiteId> = List::new();
    assert_eq!(lst.len(), 0);
    assert!(lst.is_empty());
}

#[test]
fn test_is_empty() {
    let mut list = List::new();
    assert!(list.is_empty());

    let o = list.insert_index(0, 0, 0);
    list.apply(o);
    assert!(!list.is_empty());
}

#[test]
fn test_append() {
    let mut list = List::new();
    assert!(list.is_empty());

    let op = list.append('a', 0);
    list.apply(op);
    let op = list.append('b', 0);
    list.apply(op);
    let op = list.append('c', 0);
    list.apply(op);

    assert_eq!(String::from_iter(list), "abc");
}

#[test]
fn test_out_of_order_inserts() {
    let mut list1 = List::new();
    let mut list2 = List::new();
    let o1 = list1.insert_index(0, 'a', 0);
    list1.apply(o1.clone());

    let o2 = list1.insert_index(1, 'c', 0);
    list1.apply(o2.clone());

    let o3 = list1.insert_index(1, 'b', 0);
    list1.apply(o3.clone());

    let mut operations = vec![o1, o2, o3];
    let mut iterations = 0;
    while let Some(op) = operations.pop() {
        assert!(iterations < (3 * (3 + 1)) / 2);
        iterations += 1;
        if list2.validate(&op).is_ok() {
            list2.apply(op)
        } else {
            operations.insert(0, op);
        }
    }

    let list1_elems = String::from_iter(list1);
    assert_eq!(list1_elems, "abc");
    assert_eq!(list1_elems, String::from_iter(list2));
}

#[test]
fn test_concurrent_inserts() {
    let mut list1 = List::new();
    let mut list2 = List::new();
    let mut list3 = List::new();

    let o1 = list1.insert_index(0, 'a', 'A');
    let o2 = list2.insert_index(0, 'b', 'B');

    list1.apply(o1.clone());
    list1.apply(o2.clone());
    list2.apply(o1.clone());
    list2.apply(o2.clone());
    list3.apply(o1);
    list3.apply(o2);

    assert_eq!(list1.read::<String>(), "ab");
    assert_eq!(list2.read::<String>(), "ab");
    assert_eq!(list3.read::<String>(), "ab");
    list3.apply(list3.insert_index(1, 'c', 'C'));
    assert_eq!(list3.read::<String>(), "acb");
}

#[test]
fn test_append_and_inserts() {
    let mut list = List::new();
    let o1 = list.append('a', 0);
    list.apply(o1);

    let o2 = list.insert_index(0, 'b', 0);
    list.apply(o2);

    let o3 = list.append('c', 0);
    list.apply(o3);

    let o4 = list.insert_index(1, 'd', 0);
    list.apply(o4);

    assert_eq!(String::from_iter(list), "bdac");
}

#[test]
fn test_delete_of_index() {
    let mut list = List::new();
    let o1 = list.insert_index(0, 'a', 0);
    list.apply(o1);
    let o2 = list.insert_index(1, 'b', 0);
    list.apply(o2);
    assert_eq!(String::from_iter(list.iter()), "ab");

    let op = list.delete_index(0, 0);
    list.apply(op.unwrap());
    assert_eq!(String::from_iter(list), "b");
}

#[test]
fn test_position() {
    let mut list = List::new();
    let op = list.append('a', 0);
    list.apply(op);
    let op = list.append('b', 0);
    list.apply(op);

    assert_eq!(list.pos(0), Some(&'a'));
    assert_eq!(list.pos(1), Some(&'b'));
}

#[test]
fn test_identifier_position() {
    let mut list = List::new();
    let o1 = list.append('a', 0);
    list.apply(o1.clone());
    let o2 = list.append('b', 0);
    list.apply(o2.clone());
    let o3 = list.append('c', 0);

    assert_eq!(list.pos_entry(o1.id()), Some(0));
    assert_eq!(list.pos_entry(o2.id()), Some(1));
    assert_eq!(list.pos_entry(o3.id()), None);
}

#[test]
fn test_reapply_list() {
    let mut random = rand::thread_rng();

    let s1 = random.clone().sample_iter(Alphanumeric).map(char::from);

    let mut list1 = List::new();
    let mut list2 = List::new();

    for c in s1.take(5000) {
        let index = random.gen_range(0..list1.len() + 1);
        let insert = list1.insert_index(index, c, 0);
        list1.apply(insert.clone());

        list2.apply(insert.clone());
        list2.apply(insert.clone());

        let delete = list2.delete_index(index, 1).unwrap();
        list2.apply(delete.clone());
        list2.apply(delete.clone());
        list1.apply(delete.clone());
        list1.apply(delete);

        list1.apply(insert.clone());
    }

    assert!(
        list1.is_empty(),
        "list1 was not empty: {}",
        String::from_iter(list1)
    );
    assert!(
        list2.is_empty(),
        "list2 was not empty: {}",
        String::from_iter(list2)
    );

    assert_eq!(String::from_iter(list2), String::from_iter(list1));
}

#[test]
fn test_insert_followed_by_deletes() {
    let mut random = rand::thread_rng();

    let s1 = random.clone().sample_iter(Alphanumeric).map(char::from);

    let mut list1 = List::new();
    let mut list2 = List::new();

    for c in s1.take(5000) {
        let index = random.gen_range(0..list1.len() + 1);
        let insert = list1.insert_index(index, c, 0);
        list1.apply(insert.clone());
        list2.apply(insert);

        let delete = list2.delete_index(index, 1).unwrap();
        list2.apply(delete.clone());
        list1.apply(delete);
    }

    assert!(
        list1.is_empty(),
        "list1 was not empty: {}",
        String::from_iter(list1)
    );
    assert!(
        list2.is_empty(),
        "list2 was not empty: {}",
        String::from_iter(list2)
    );
}

#[test]
fn test_mutual_insert() {
    let mut list1 = List::new();
    let mut list2 = List::new();
    let plan = vec![
        (4, 42, false),
        (22, 5, true),
        (1, 44, false),
        (23, 88, false),
        (99, 3, true),
    ];

    for (e, i, s) in plan {
        let ((source, source_actor), replica) = if s {
            ((&mut list1, 0), &mut list2)
        } else {
            ((&mut list2, 1), &mut list1)
        };
        let i = i % (source.len() + 1);
        println!("{:?} inserting {} @ {}", source_actor, e, i);
        let op = source.insert_index(i, e, source_actor);
        source.apply(op.clone());
        replica.apply(op);
    }

    assert_eq!(Vec::from_iter(list1), Vec::from_iter(list2));
}

#[test]
fn test_deep_inserts() {
    let mut list = List::new();
    let mut vec = Vec::new();
    let n = 1000;
    for v in 0..n {
        let i = list.len() / 2;
        println!("inserting {}/{}", i, list.len());
        vec.insert(i, v);
        let op = list.insert_index(i, v, 0);
        list.apply(op);
    }
    assert_eq!(list.len(), n);
    assert_eq!(Vec::from_iter(list), vec);
}
