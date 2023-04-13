use buddle_object_property::{
    Type,
    cpp::RawString,
    serde::{
        Deserializer,
        Config,
        SerializerFlags,
    },
    registry::Registry,
};
use buddle_wad::{Archive, Interner};

#[derive(Clone, Debug, Default, Type)]
pub struct TemplateManifest {
    #[property(flags(SAVE | COPY | PUBLIC))]
    serialized_templates: Vec<TemplateLocation>,
}

#[derive(Clone, Debug, Default, Type)]
pub struct TemplateLocation {
    #[property(flags(SAVE | COPY | PUBLIC))]
    pub filename: RawString,
    #[property(flags(SAVE | COPY | PUBLIC))]
    pub id: u32,
}

fn main() {
    let root = Archive::heap("Root.wad", false).unwrap();
    let mut intern = Interner::new(&root);

    let handle = intern.intern("TemplateManifest.xml").unwrap();
    let template_manifest = intern.fetch(handle).unwrap();

    let mut config = Config::new();
    config.shallow = false;
    config.flags = SerializerFlags::STATEFUL_FLAGS;

    let mut registry = Registry::new();

    registry.register::<TemplateManifest>();

    let mut derserilizer = Deserializer::new(config, &registry);

    let mut garbage = Vec::new();
    derserilizer.load(template_manifest, &mut garbage).unwrap();
    let manifest = derserilizer.deserialize().unwrap();

    println!("{manifest:?}");
}
