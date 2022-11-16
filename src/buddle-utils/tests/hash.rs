use buddle_utils::hash::*;

#[test]
fn test_djb2() {
    assert_eq!(djb2("m_packedName"), 307420154);
}

#[test]
fn test_string_id() {
    assert_eq!(string_id("std::string"), 1497788074);
    assert_eq!(string_id("class FishTournamentEntry"), 1725212200);
    assert_eq!(
        StringIdBuilder::new()
            .feed_str("class ")
            .feed_str("NonCombatMayCastSpellTemplate")
            .feed_str("")
            .feed_str("*")
            .finish(),
        920052956
    );
}
