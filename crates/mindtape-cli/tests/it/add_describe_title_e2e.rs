//! `mt add --describe` takes its format after `=`, so a word beside it lands
//! in the title and used to describe the default template without a word. It
//! is refused instead, naming the template when the word is one.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

fn refusal(dir: &std::path::Path, args: &[&str]) -> String {
    let out = support::mt(dir)
        .arg("add")
        .args(args)
        .assert()
        .failure()
        .code(2);
    String::from_utf8_lossy(&out.get_output().stderr).into_owned()
}

#[test]
fn a_template_name_beside_describe_suggests_dash_t() {
    let dir = support::scaffold();
    std::fs::create_dir_all(dir.path().join(".mindtape/templates")).unwrap();
    std::fs::write(
        dir.path().join(".mindtape/templates/finding.typ"),
        "#import \"@local/mindtape:0.2.0\": *\n#show: task.with(title: slot(\"title\"))\n",
    )
    .unwrap();
    let stderr = refusal(dir.path(), &["--describe", "finding"]);
    assert_eq!(
        stderr,
        "error: --describe takes no title: 'finding'\n\
         hint: to describe that template, name it with -t: mt add -t finding --describe\n"
    );
}

#[test]
fn any_other_title_beside_describe_is_refused() {
    let dir = support::scaffold();
    let stderr = refusal(dir.path(), &["--describe", "Fix", "the", "bug"]);
    assert_eq!(
        stderr,
        "error: --describe takes no title: 'Fix the bug'\n\
         hint: to describe a template, name it with -t\n"
    );
}

#[test]
fn describe_alone_and_with_a_template_still_describes() {
    let dir = support::scaffold();
    support::mt(dir.path())
        .args(["add", "--describe", "-t", "task"])
        .assert()
        .success();
}
