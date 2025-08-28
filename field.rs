use std::cell::RefCell;
use std::rc::{Rc, Weak};

// TODO: these can be RwLocks for thread safety
type WeakFlag = Weak<RefCell<bool>>;
type RcFlag = Rc<RefCell<bool>>;

pub struct Field<T, F>
where
    F: Fn() -> T,
{
    value: Value<T, F>,

    /// Clean flags for values which depend on this field.
    dependents: Vec<WeakFlag>,
}

impl<T, F> Field<T, F>
where
    F: Fn() -> T,
{
    /// Mark all existing dependents as dirty and remove nonexisting ones
    pub fn invalidate_downstream(&mut self) {
        self.dependents.retain(|weak| {
            let opt = weak.upgrade();
            if let Some(rc) = &opt {
                *rc.borrow_mut() = false;
            }
            opt.is_some()
        })
    }

    pub fn set(&mut self, new_val: T) {
        self.value.set(new_val);
        self.invalidate_downstream();
    }

    pub fn new(value: Value<T, F>) -> Self {
        Field {
            value,
            dependents: vec![],
        }
    }

    /// Construct a new known field with no dependents.
    pub fn known(val: T) -> Self {
        Field::new(Value::Known(val))
    }

    /// Create a new shared flag for use in computed fields.
    pub fn get_shared_flag(&mut self) -> RcFlag {
        // TODO: When is this useful?
        // Always start as dirty; compute lazily on first get
        let flag = Rc::new(RefCell::new(false));
        self.dependents.push(Rc::downgrade(&flag));
        flag
    }

    pub fn add_dependent(&mut self, flag: WeakFlag) {
        self.dependents.push(flag)
    }
}

pub enum Value<T, F>
where
    F: Fn() -> T,
{
    Known(T),
    Computed(Computed<T, F>),
}

impl<T, F> Value<T, F>
where
    F: Fn() -> T,
{
    pub fn get(&mut self) -> &T {
        match self {
            Value::Known(val) => val,
            Value::Computed(comp) => comp.get(),
        }
    }

    pub fn set(&mut self, new_val: T) {
        match self {
            Value::Known(val) => *val = new_val,
            Value::Computed(val) => val.set(new_val),
        }
    }
}

pub struct Computed<T, F>
where
    F: Fn() -> T,
{
    compute_fn: F,

    // TODO: make this an Option<T> for new instances
    cache: T,
    clean: Rc<RefCell<bool>>,
}

impl<T, F> Computed<T, F>
where
    F: Fn() -> T,
{
    // TODO: make a constructor which accepts some fields and a function over them!
    // Maybe use tuples overloading (A), (A,B), etc. like Bevy?

    pub fn get(&mut self) -> &T {
        if !(*self.clean.borrow()) {
            self.cache = (self.compute_fn)();
            *self.clean.borrow_mut() = true;
        }
        &self.cache
    }

    pub fn set(&mut self, new_val: T) {
        // TODO: this could overwrite with a wrong value; is this even valid?
        self.cache = new_val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn very_manual_connection() {
        let known: Field<_, fn() -> bool> = Field::known(true);
        let known_rc = Rc::new(RefCell::new(known));

        let clean_flag = Rc::new(RefCell::new(false));
        known_rc
            .borrow_mut()
            .add_dependent(Rc::downgrade(&clean_flag));

        let compute_fn = {
            let known_rc = Rc::clone(&known_rc);
            move || *known_rc.borrow_mut().value.get()
        };

        // Just mirror the known field
        let mut computed = Field {
            value: Value::Computed(Computed {
                compute_fn,
                cache: false,
                clean: clean_flag,
            }),
            dependents: vec![],
        };

        assert_eq!(*computed.value.get(), true);

        known_rc.borrow_mut().set(false);

        assert_eq!(*computed.value.get(), false);
    }
}
