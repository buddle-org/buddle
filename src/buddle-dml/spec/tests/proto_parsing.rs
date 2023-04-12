use std::{fs::read_to_string, path::PathBuf};

use buddle_dml_spec::{parse_protocol, Protocol};

fn read_test_proto() -> anyhow::Result<Protocol> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/TestProto.xml");

    let proto = read_to_string(path)?;
    parse_protocol(&proto)
}

#[test]
fn basic_messages() -> anyhow::Result<()> {
    let proto = read_test_proto()?;

    assert!(proto.record("MSG_A").is_some());
    assert!(proto.record("MSG_B").is_some());

    assert_eq!(proto.protocol_info().map(|r| r.message_order()), Some(0));
    assert_eq!(proto.record("MSG_A").map(|r| r.message_order()), Some(1));
    assert_eq!(proto.record("MSG_B").map(|r| r.message_order()), Some(2));

    assert_eq!(proto.record("MSG_A").and_then(|r| r.message_handler()), Some("a"));
    assert_eq!(proto.record("MSG_B").and_then(|r| r.message_handler()), Some("b"));

    assert_eq!(proto.protocol_info().and_then(|r| r.service_id()), Some(1));

    Ok(())
}
