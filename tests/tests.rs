#![expect(unused)]

use std::sync::Arc;

use forgy::Build;

#[test]
fn derives_on_unit() {
    #[derive(Build)]
    struct Unit;

    let mut c = forgy::Container::new(());

    let _: Arc<Unit> = c.get();
}

#[test]
fn derives_on_tuple() {
    #[derive(Build)]
    struct Unit;

    #[derive(Build)]
    struct Tuple(Arc<Unit>);

    let mut c = forgy::Container::new(());

    let _: Arc<Tuple> = c.get();
}

#[test]
fn derives_on_struct() {
    #[derive(Build)]
    struct Unit;

    #[derive(Build)]
    struct Struct {
        unit: Arc<Unit>,
    }

    let mut c = forgy::Container::new(());

    let _: Arc<Struct> = c.get();
}

#[test]
fn derives_with_input() {
    struct Input {
        string: String,
    }

    #[derive(Build)]
    #[forgy(input = Input)]
    struct Struct {
        #[forgy(value = input.string.clone())]
        from_input: String,
    }

    let mut c = forgy::Container::new(Input {
        string: "some string".to_string(),
    });

    let s: Arc<Struct> = c.get();
    assert_eq!(s.from_input, "some string");
}

#[test]
fn derives_with_input_and_ambivalend() {
    struct Input {
        string: String,
    }

    #[derive(Build)]
    #[forgy(input = Input)]
    struct Struct {
        #[forgy(value = input.string.clone())]
        from_input: String,
        dep: Arc<Dep>,
    }

    #[derive(Build)]
    struct Dep;

    let mut c = forgy::Container::new(Input {
        string: "some string".to_string(),
    });

    let s: Arc<Struct> = c.get();
    assert_eq!(s.from_input, "some string");
}

#[test]
fn constructs_default_values() {
    #[derive(Build)]
    struct Struct {
        #[forgy(value = Default::default())]
        some_cache: std::collections::HashSet<String>,
    }

    let mut c = forgy::Container::new(());

    let _: Arc<Struct> = c.get();
}

#[test]
fn constructs_with_value() {
    #[derive(Build)]
    struct Struct {
        #[forgy(value = 16)]
        max_tasks: u32,
    }

    let mut c = forgy::Container::new(());

    let s: Arc<Struct> = c.get();
    assert_eq!(s.max_tasks, 16);
}
