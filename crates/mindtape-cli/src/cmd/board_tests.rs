use super::*;

#[test]
fn the_footer_names_the_set_and_the_hidden_rows_only_when_some_hide() {
    assert_eq!(footer(3, 0, Some("home")), "@home · 3 projects");
    assert_eq!(
        footer(3, 2, Some("home")),
        "@home · 3 projects · 2 hidden · mt @home board --all"
    );
    assert_eq!(footer(1, 1, None), "1 project · 1 hidden · mt board --all");
}

#[test]
fn counts_align_right_and_the_last_column_is_not_padded() {
    let widths = [5, 4, 3];
    let right = [false, true, false];
    let line = |texts: &[&str]| -> Vec<String> { texts.iter().map(|t| (*t).to_owned()).collect() };
    assert_eq!(
        layout(&line(&["ab", "7", "x"]), &widths, &right),
        "ab        7  x"
    );
    assert_eq!(
        layout(&line(&["⚠ c", "it broke"]), &widths, &right),
        "⚠ c    it broke"
    );
}

#[test]
fn a_cell_prints_its_count_task_or_question_mark() {
    let hashes = StemHashes::new(["one"]);
    let top = mindtape::TopTask {
        id: "one".to_owned(),
        title: "One".to_owned(),
        file: PathBuf::from("tasks/one.typ"),
    };
    let hash = hashes.hash_of("one").unwrap_or_default().to_owned();
    assert_eq!(cell_text(&Cell::Count(4), &hashes), "4");
    assert_eq!(
        cell_text(&Cell::Top(Some(top)), &hashes),
        format!("{hash} One")
    );
    assert_eq!(cell_text(&Cell::Top(None), &hashes), "-");
    assert_eq!(cell_text(&Cell::Failed("no".to_owned()), &hashes), "?");
}
