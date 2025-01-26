//! Derive macro and traits for building dependency graphs. Simple dependency injector.
//!
//! ```
//! use std::sync::Arc;
//!
//! #[derive(forgy::Build)]
//! #[forgy(input = Input)]
//! struct Foo {
//!   #[forgy(value = input.the_string.clone())]
//!   from_input: String,
//! }
//!
//! #[derive(forgy::Build)]
//! #[forgy(input = Input)]
//! struct Bar {
//!   foo: Arc<Foo>,
//! }
//!
//! struct Input {
//!   the_string: String,
//! }
//!
//! fn main() {
//!   let mut c = forgy::Container::new(Input {
//!     the_string: "from input".to_string(),
//!   });
//!
//!   let bar: Bar = c.build();
//!   assert_eq!(bar.foo.from_input, "from input");
//!
//!   let foo: Arc<Foo> = c.get();
//!   assert_eq!(Arc::as_ptr(&bar.foo), Arc::as_ptr(&foo));
//! }
//! ```
use std::{any::TypeId, sync::Arc};

pub use forgy_derive::Build;

/// A type that can be constructed given the [Container].
pub trait Build<I = ()>: 'static {
    fn build(container: &mut Container<I>) -> Self;
}

/// A container for constructed objects.
pub struct Container<I = ()> {
    input: I,
    built: anymap::AnyMap,

    stack: Vec<TypeId>,
}

impl<I> Container<I> {
    /// Construct a new Container with the provided input.
    pub fn new(input: I) -> Container<I> {
        Container {
            input,
            built: anymap::AnyMap::new(),
            stack: Vec::new(),
        }
    }

    /// Get a reference to the provided input.
    pub fn input(&self) -> &I {
        &self.input
    }

    /// Get the already created T, or build and store a new T.
    pub fn get<T: Build<I>>(&mut self) -> Arc<T> {
        if let Some(got) = self.built.get::<Arc<T>>() {
            return Arc::clone(&got);
        }

        let new = Arc::new(self.build());
        self.built.insert(Arc::clone(&new));
        new
    }

    /// Build and do not store a new T.
    pub fn build<T: Build<I>>(&mut self) -> T {
        let type_id = TypeId::of::<T>();
        if self.stack.contains(&type_id) {
            panic!("Cycle constructing {type_id:?}: {:?}", self.stack);
        }

        self.stack.push(type_id);
        let new = T::build(self);
        self.stack.pop();

        new
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU8;

    use super::*;

    struct Unit;

    impl Build for Unit {
        fn build(_: &mut Container) -> Self {
            Unit
        }
    }

    struct Counter(u8);

    impl Build for Counter {
        fn build(_: &mut Container) -> Self {
            static CONSTRUCTED: AtomicU8 = AtomicU8::new(0);
            Counter(CONSTRUCTED.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        }
    }

    struct HasDep {
        #[expect(unused)]
        unit: Arc<Unit>,
    }

    impl Build for HasDep {
        fn build(constructor: &mut Container) -> Self {
            HasDep {
                unit: constructor.get(),
            }
        }
    }

    struct GenericDep<T> {
        #[expect(unused)]
        dep: Arc<T>,
    }

    impl<T> Build for GenericDep<T>
    where
        T: Build,
    {
        fn build(constructor: &mut Container) -> Self {
            GenericDep {
                dep: constructor.get(),
            }
        }
    }

    #[test]
    fn unit_value() {
        let mut c = Container::new(());

        let _: Arc<Unit> = c.get::<Unit>();
    }

    #[test]
    fn reuses_previous_constructed_values() {
        let mut c = Container::new(());

        let first: Arc<Counter> = c.get::<Counter>();
        let second: Arc<Counter> = c.get::<Counter>();

        assert_eq!(first.0, second.0);
    }

    #[test]
    fn constructs_with_dependency() {
        let mut c = Container::new(());

        let _: Arc<HasDep> = c.get();
    }

    #[test]
    fn works_with_generic_dep() {
        let mut c = Container::new(());

        let _: Arc<GenericDep<Unit>> = c.get();
    }

    #[test]
    #[should_panic]
    fn panics_with_cycle() {
        let mut c = Container::new(());

        #[expect(unused)]
        struct Foo(Arc<Bar>);

        impl Build for Foo {
            fn build(constructor: &mut Container) -> Self {
                Foo(constructor.get())
            }
        }

        #[expect(unused)]
        struct Bar(Arc<Foo>);

        impl Build for Bar {
            fn build(constructor: &mut Container) -> Self {
                Bar(constructor.get())
            }
        }

        let _: Arc<Foo> = c.get();
    }

    struct Config {
        string: String,
    }

    #[test]
    fn can_get_values_from_configuration() {
        let mut c = Container::new(Config {
            string: "some string".to_string(),
        });

        struct Dep {
            string_from_config: String,
        }

        impl Build<Config> for Dep {
            fn build(constructor: &mut Container<Config>) -> Self {
                Dep {
                    string_from_config: constructor.input().string.clone(),
                }
            }
        }

        let dep: Arc<Dep> = c.get();
        assert_eq!(dep.string_from_config, "some string");
    }
}
