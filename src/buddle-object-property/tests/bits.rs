use buddle_object_property::{bitenum, Enum};

bitenum! {
    pub struct Flags {
        const A = 1 << 0;
        const B = 1 << 1;
        const C = 1 << 2;
    }
}

#[test]
fn bit_reflect() {
    let mut flags = Flags::A | Flags::C;

    assert_eq!(flags.variant(), "A | C");
    assert_eq!(flags.value(), 0b101);

    flags.update_variant("A | B");
    assert_eq!(flags.variant(), "A | B");
    assert_eq!(flags.value(), 0b11);
}
