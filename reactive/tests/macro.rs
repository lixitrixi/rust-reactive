use rust_reactive::prelude::*;

#[test]
fn make_producer() {
    let x = rv!(0);
    assert_eq!(x.get(), 0);
}

#[test]
fn single_dependency_bool() {
    let mut a = rv!(true);
    let b = rv!(!a);

    assert!(!b.get());
    a.set(false);
    assert!(b.get());
}

#[test]
fn two_dependencies_int() {
    let mut a = rv!(0);
    let mut b = rv!(0);
    let sum = rv!(a + b);

    assert_eq!(sum.get(), 0);
    a.set(1);
    assert_eq!(sum.get(), 1);
    b.set(1);
    assert_eq!(sum.get(), 2);
}
