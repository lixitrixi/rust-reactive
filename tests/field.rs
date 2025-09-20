use rust_reactive::prelude::*;

#[test]
fn single_dependent_computed() {
    let mut known = Field::new((), || false);
    let computed = Field::new(&known, |b| b);

    assert!(!computed.get());
    known.set(true);
    assert!(computed.get());
}

#[test]
fn single_transitive_dependent() {
    let mut known = Field::new((), || false);
    let computed1 = Field::new(&known, |b| b);
    let computed2 = Field::new(&computed1, |b| b);

    assert!(!computed2.get());
    known.set(true);
    assert!(computed2.get());
}

#[test]
fn multiple_dependents() {
    let mut bool_a = Field::new((), || false);
    let mut bool_b = Field::new((), || false);
    let xor = Field::new((&bool_a, &bool_b), |a, b| a ^ b);

    assert!(!xor.get());
    bool_a.set(true);
    assert!(xor.get());
    bool_b.set(true);
    assert!(!xor.get());
}
