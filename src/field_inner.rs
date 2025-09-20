use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct FieldInner<T: Copy> {
    value: Option<T>,
    producer: Box<dyn Fn() -> T>,
    depen: Rc<RefCell<Depen>>,
}

impl<T: Copy> FieldInner<T> {
    pub fn new(producer: Box<dyn Fn() -> T>) -> Self {
        FieldInner {
            value: None,
            producer,
            depen: Rc::new(RefCell::new(Depen::new())),
        }
    }

    pub fn get(&mut self) -> T {
        self.refresh();
        self.value
            .expect("field should contain a value after refresh")
    }

    pub fn set(&mut self, val: T) {
        // TODO: semantics when this is a computed value?
        // Idea: drop the current depen to cut off from dependencies
        self.value = Some(val);
        let mut depen = self.depen.borrow_mut();
        depen.invalidate_dependents();
        depen.set_clean(true);
    }

    fn refresh(&mut self) {
        if self.depen.borrow().is_dirty() {
            self.value = Some((self.producer)());
            self.depen.borrow_mut().set_clean(true);
        }
    }

    pub fn add_dependent<S: Copy>(&self, other: &FieldInner<S>) {
        self.depen
            .borrow_mut()
            .add_dependent(Rc::downgrade(&other.depen));
    }
}

/// A node in a dependency graph.
struct Depen {
    clean: bool,
    dependents: Vec<Weak<RefCell<Depen>>>,
}

impl Depen {
    pub fn new() -> Self {
        Depen {
            clean: false,
            dependents: vec![],
        }
    }

    pub fn is_dirty(&self) -> bool {
        !self.clean
    }

    pub fn set_clean(&mut self, clean: bool) {
        self.clean = clean;
    }

    pub fn invalidate(&mut self) {
        self.clean = false;
        self.invalidate_dependents();
    }

    pub fn invalidate_dependents(&mut self) {
        self.dependents.retain(|weak| {
            let opt = weak.upgrade();
            if let Some(dep) = &opt {
                dep.borrow_mut().invalidate();
            }
            opt.is_some()
        });
    }

    pub fn add_dependent(&mut self, dep: Weak<RefCell<Depen>>) {
        self.dependents.push(dep);
    }
}
