use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::{field::Field, field_inner::FieldInner};

fn val<V: Copy>(weak: &Weak<RefCell<FieldInner<V>>>) -> V {
    let rc = weak.upgrade().expect("dependency has been dropped");
    rc.borrow_mut().get()
}

pub trait Dependencies<T> {
    type Compute;

    fn make_producer(&self, compute: Self::Compute) -> Box<dyn Fn() -> T>;

    fn register_dependent<U: Copy>(&self, dependent: &Field<U>);
}

impl<T> Dependencies<T> for ()
where
    T: 'static,
{
    type Compute = fn() -> T;

    fn make_producer(&self, compute: Self::Compute) -> Box<dyn Fn() -> T> {
        Box::new(compute)
    }

    fn register_dependent<U: Copy>(&self, _: &Field<U>) {}
}

impl<T, V1> Dependencies<T> for &Field<V1>
where
    T: 'static,
    V1: Copy + 'static,
{
    type Compute = fn(V1) -> T;

    fn make_producer(&self, compute: Self::Compute) -> Box<dyn Fn() -> T> {
        let v1_weak = Rc::downgrade(&self.inner);
        Box::new(move || {
            let v1 = val(&v1_weak);
            compute(v1)
        })
    }

    fn register_dependent<U: Copy>(&self, dependent: &Field<U>) {
        self.add_dependent(dependent);
    }
}

impl<T, V1, V2> Dependencies<T> for (&Field<V1>, &Field<V2>)
where
    T: 'static,
    V1: Copy + 'static,
    V2: Copy + 'static,
{
    type Compute = fn(V1, V2) -> T;

    fn make_producer(&self, compute: Self::Compute) -> Box<dyn Fn() -> T> {
        let v1_weak = Rc::downgrade(&self.0.inner);
        let v2_weak = Rc::downgrade(&self.1.inner);
        Box::new(move || {
            let (v1, v2) = (val(&v1_weak), val(&v2_weak));
            compute(v1, v2)
        })
    }

    fn register_dependent<U: Copy>(&self, dependent: &Field<U>) {
        self.0.add_dependent(dependent);
        self.1.add_dependent(dependent);
    }
}

impl<T, V1, V2, V3> Dependencies<T> for (&Field<V1>, &Field<V2>, &Field<V3>)
where
    T: 'static,
    V1: Copy + 'static,
    V2: Copy + 'static,
    V3: Copy + 'static,
{
    type Compute = fn(V1, V2, V3) -> T;

    fn make_producer(&self, compute: Self::Compute) -> Box<dyn Fn() -> T> {
        let v1_weak = Rc::downgrade(&self.0.inner);
        let v2_weak = Rc::downgrade(&self.1.inner);
        let v3_weak = Rc::downgrade(&self.2.inner);
        Box::new(move || {
            let (v1, v2, v3) = (val(&v1_weak), val(&v2_weak), val(&v3_weak));
            compute(v1, v2, v3)
        })
    }

    fn register_dependent<U: Copy>(&self, dependent: &Field<U>) {
        self.0.add_dependent(dependent);
        self.1.add_dependent(dependent);
        self.2.add_dependent(dependent);
    }
}
