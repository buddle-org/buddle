use buddle_object_property::{
    type_info::{PropertyFlags, TypeInfo},
    Enum, PropertyClass, PropertyClassExt, Type,
};

#[test]
fn basic_reflection() {
    #[derive(Default, Type)]
    pub struct Foo {
        #[property()]
        a: u32,
        #[property(name = "c")]
        b: i32,
    }

    let mut x: Box<dyn PropertyClass> = Box::new(Foo { a: 7, b: -5 });
    let list = x.property_list();

    assert!(x.base().is_none());

    let a_access = list.property("a").unwrap();
    let a = x.property(a_access).unwrap();
    assert!(a.is::<u32>());
    assert_eq!(a.downcast_ref(), Some(&7_u32));

    let b_access = list.property("c").unwrap();
    assert_eq!(x.property_as::<i32>(b_access), Some(&-5));
    x.property_mut(b_access).unwrap().set(Box::new(-6_i32)).ok();
    assert_eq!(x.property_as::<i32>(b_access), Some(&-6));
}

#[test]
fn custom_type_info() {
    const U32: TypeInfo = TypeInfo::leaf::<u32>(Some("u32"));

    #[derive(Default, Type)]
    pub struct Foo {
        #[property()]
        a: u32,
        #[property(info = &U32)]
        b: u32,
    }

    let x: Box<dyn PropertyClass> = Box::new(Foo { a: 1, b: 2 });
    let list = x.property_list();

    assert_eq!(
        list.property("a").map(|p| p.type_info().type_name()),
        Some("unsigned int")
    );
    assert_eq!(
        list.property("b").map(|p| p.type_info().type_name()),
        Some("u32")
    );
}

#[test]
fn fake_inheritance() {
    #[derive(Clone, Copy, Default, Type)]
    pub struct A {
        test: u32,
    }

    #[derive(Clone, Copy, Default, Type)]
    pub struct B {
        #[property(base)]
        base: A,
    }

    #[derive(Clone, Copy, Default, Type)]
    pub struct C {
        #[property(base)]
        base: B,
    }

    let b = B {
        base: A { test: 5 },
    };

    let c: Box<dyn PropertyClass> = Box::new(C { base: b });
    let b: Box<dyn PropertyClass> = Box::new(b);

    assert_eq!(
        b.base_as::<A>().unwrap().test,
        c.base_as::<A>().unwrap().test
    );
}

#[test]
fn property_flags() {
    #[derive(Default, Type)]
    pub struct Foo {
        #[property(flags(PUBLIC | COPY))]
        a: u32,
        #[property()]
        b: u32,
    }

    let foo: Box<dyn PropertyClass> = Box::<Foo>::default();
    let list = foo.property_list();

    assert_eq!(
        list.property("a").map(|a| a.flags()),
        Some(PropertyFlags::PUBLIC | PropertyFlags::COPY)
    );
    assert_eq!(
        list.property("b").map(|a| a.flags()),
        Some(PropertyFlags::empty())
    );
}

#[test]
fn object_name() {
    #[derive(Default, Type)]
    #[object(name = "Bar")]
    pub struct Foo {}

    let foo: Box<dyn PropertyClass> = Box::new(Foo {});
    assert_eq!(foo.property_list().type_name(), "Bar");
}

#[test]
fn enum_reflection() {
    #[derive(Debug, PartialEq, Type)]
    pub enum Foo {
        A = 1,
        B = 2,
    }

    assert_eq!(Foo::from_variant("A"), Some(Foo::A));
    assert_eq!(Foo::B.value(), 2);
}
