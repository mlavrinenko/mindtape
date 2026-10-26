use std::path::PathBuf;

use super::strip_verbatim;

#[test]
fn a_verbatim_prefix_comes_off_and_a_plain_path_stays() {
    for (path, shown) in [
        (r"\\?\C:\work\team.toml", r"C:\work\team.toml"),
        (r"\\?\UNC\host\share\team.toml", r"\\host\share\team.toml"),
        ("/work/team.toml", "/work/team.toml"),
    ] {
        assert_eq!(strip_verbatim(PathBuf::from(path)), PathBuf::from(shown));
    }
}
