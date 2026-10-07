use super::key_line;

const TEXT: &str = "extends = [\"x\"]\n\n[views.broken]\nsort = [\"a\"]\n\n[[on.flip]]\nwhen = \"w\"\n\n[[on.flip]]\nactions = [{ run = \"true\" }]\n";

#[test]
fn a_key_resolves_to_its_own_line() {
    assert_eq!(key_line(TEXT, "extends"), Some(1));
    assert_eq!(key_line(TEXT, "views.broken.sort"), Some(4));
    assert_eq!(key_line(TEXT, "views.broken"), Some(3));
}

#[test]
fn an_array_index_resolves_to_its_entry() {
    assert_eq!(key_line(TEXT, "on.flip[1]"), Some(9));
    assert_eq!(key_line(TEXT, "on.flip[1].actions"), Some(10));
    assert_eq!(key_line(TEXT, "on.flip[1].actions[0]"), Some(10));
}

#[test]
fn an_undeclared_tail_falls_back_to_its_deepest_ancestor() {
    assert_eq!(key_line(TEXT, "views.broken.filter"), Some(3));
    assert_eq!(key_line(TEXT, "nothing"), None);
}
