use std::cell::RefCell;
use std::rc::Rc;

use crate::field_inner::FieldInner;
use crate::traits::Dependencies;

pub struct Field<T: Copy> {
    pub(crate) inner: Rc<RefCell<FieldInner<T>>>,
}

impl<T: Copy> Field<T> {
    pub fn new<D>(dependencies: D, compute_fn: D::Compute) -> Field<T>
    where
        D: Dependencies<T>,
    {
        let producer = dependencies.make_producer(compute_fn);
        let field = Field {
            inner: Rc::new(RefCell::new(FieldInner::new(producer))),
        };
        dependencies.register_dependent(&field);
        field
    }

    pub fn add_dependent<U: Copy>(&self, dependent: &Field<U>) {
        let dep_inner = dependent.inner.borrow();
        self.inner.borrow_mut().add_dependent(&dep_inner);
    }

    pub fn get(&self) -> T {
        self.inner.borrow_mut().get()
    }

    pub fn set(&mut self, val: T) {
        self.inner.borrow_mut().set(val);
    }
}
