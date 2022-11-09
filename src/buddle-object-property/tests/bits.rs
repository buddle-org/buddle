use buddle_object_property::{bitenum, Enum};

bitenum! {
    pub struct Flags {
        const A = 1 << 0;
        const B = 1 << 1;
        const C = 1 << 2;
    }
}

#[test]
fn simple_reflect() {
    let flags = Flags::A | Flags::C;

    assert_eq!(flags.variant(), "A | C");
    assert_eq!(flags.value(), 0b101);
    assert_eq!(Flags::from_variant("A | C"), Some(flags));
    assert_eq!(Flags::from_value(0b101), Some(flags));
}
