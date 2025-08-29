use std::cell::{Cell, Ref, RefCell};
use std::rc::{Rc, Weak};

pub struct Field<'a, T> {
    cache: Rc<RefCell<Option<T>>>,

    compute_fn: Box<dyn Fn() -> T + 'a>,

    /// A flag indicating whether the currently cached value is still valid.
    /// Dependencies are given a weak ref to this field to set when they change.
    clean: Rc<Cell<bool>>,

    /// Clean flags for fields which depend on this field.
    dependents: Vec<Weak<Cell<bool>>>,
}

impl<'a, T> Field<'a, T> {
    pub fn get(&self) -> Ref<T> {
        self.refresh();
        Ref::map(self.cache.borrow(), |v| {
            v.as_ref().expect("Value empty after refresh")
        })
    }

    pub fn set(&mut self, new_val: T) {
        *self.cache.borrow_mut() = Some(new_val);
        self.invalidate_downstream();
    }

    /// If dirty, recompute the cache value.
    fn refresh(&self) {
        if !self.clean.get() {
            *self.cache.borrow_mut() = Some((self.compute_fn)());
            self.clean.set(true);
        }
    }

    pub fn add_dependent(&mut self, flag: Weak<Cell<bool>>) {
        self.dependents.push(flag)
    }

    /// Mark all existing dependents as dirty and remove nonexisting ones
    pub fn invalidate_downstream(&mut self) {
        self.dependents.retain(|weak| {
            let opt = weak.upgrade();
            if let Some(rc) = &opt {
                rc.set(false);
            }
            opt.is_some()
        })
    }

    /// Given a flag to invalidate on changes, return a weak reference to this field's value.
    pub fn exchange(&mut self, flag: Weak<Cell<bool>>) -> Weak<RefCell<Option<T>>> {
        self.add_dependent(flag);

        // TODO: this allows mutating the inner value.
        Rc::downgrade(&self.cache)
    }

    /// Construct a new field with a given value and no dependencies.
    pub fn known(val: T) -> Self {
        Field {
            cache: Rc::new(RefCell::new(Some(val))),
            compute_fn: Box::new(|| panic!("Cannot compute a known value")),
            clean: Rc::new(Cell::new(true)),
            dependents: vec![],
        }
    }

    pub fn computed<'dep, D: 'a>(
        dependency: &'dep mut Field<'a, D>,
        compute_fn: impl Fn(&D) -> T + 'a,
    ) -> Self {
        // Start off dirty and compute on the first get
        let flag = Rc::new(Cell::new(false));

        let weak = dependency.exchange(Rc::downgrade(&flag));
        Field {
            cache: Rc::new(RefCell::new(None)),
            compute_fn: Box::new(move || {
                let rc = weak.upgrade().expect("Dependency has been dropped");
                let ref_opt = rc.borrow();
                let d_ref = ref_opt.as_ref().expect("Empty value used for compute");
                compute_fn(d_ref)
            }),
            clean: flag,
            dependents: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_dependent_computed() {
        let mut known = Field::known(false);
        let computed = Field::computed(&mut known, |b| *b);

        assert!(!*computed.get());
        known.set(true);
        assert!(*computed.get());
    }

    #[test]
    fn dependent_computed_in_sequence() {
        let mut known = Field::known(false);
        let mut computed1 = Field::computed(&mut known, |b| *b);
        let computed2 = Field::computed(&mut computed1, |b| *b);

        assert!(!*computed2.get());
        known.set(true);
        assert!(*computed2.get());
    }
}
