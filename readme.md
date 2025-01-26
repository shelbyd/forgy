# forgy

Derive macro and traits for building dependency graphs. Simple dependency injector.

```rust
use std::sync::Arc;

#[derive(forgy::Build)]
#[forgy(input = Input)]
struct Foo {
  #[forgy(value = input.the_string.clone())]
  from_input: String,
}

#[derive(forgy::Build)]
#[forgy(input = Input)]
struct Bar {
  foo: Arc<Foo>,
}

struct Input {
  the_string: String,
}

fn main() {
  let mut c = forgy::Container::new(Input {
    the_string: "from input".to_string(),
  });

  let bar: Bar = c.build();
  assert_eq!(bar.foo.from_input, "from input");

  let foo: Arc<Foo> = c.get();
  assert_eq!(Arc::as_ptr(&bar.foo), Arc::as_ptr(&foo));
}
```
