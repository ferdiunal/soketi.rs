use std::fs;

const README_FILES: [&str; 2] = ["README.md", "README.en.md"];
const GITHUB_REPOSITORY: &str = "ferdiunal/soketi.rs";

#[test]
fn readme_github_badges_point_to_current_repository() {
    for readme in README_FILES {
        let content = fs::read_to_string(readme).expect("README file should be readable");

        assert!(
            !content.contains("img.shields.io/github/actions/workflow/status/ferdiunal/soketi-rs/"),
            "{readme} uses the old hyphenated GitHub repository in the build badge"
        );
        assert!(
            !content.contains("img.shields.io/github/v/release/ferdiunal/soketi-rs"),
            "{readme} uses the old hyphenated GitHub repository in the release badge"
        );
        assert!(
            content.contains(&format!(
                "https://img.shields.io/github/actions/workflow/status/{GITHUB_REPOSITORY}/release.yml?branch=main"
            )),
            "{readme} should use the current GitHub repository in the build badge"
        );
        assert!(
            content.contains(&format!(
                "https://img.shields.io/github/v/release/{GITHUB_REPOSITORY}"
            )),
            "{readme} should use the current GitHub repository in the release badge"
        );
    }
}

#[test]
fn readme_github_badges_have_valid_markdown_links() {
    for readme in README_FILES {
        let content = fs::read_to_string(readme).expect("README file should be readable");

        assert!(
            !content.contains("]actions)"),
            "{readme} should not leave a literal actions suffix after a badge"
        );
        assert!(
            !content.contains("]releases)"),
            "{readme} should not leave a literal releases suffix after a badge"
        );
        assert!(
            content.contains(&format!(
                "](https://github.com/{GITHUB_REPOSITORY}/actions/workflows/release.yml)"
            )),
            "{readme} should link the build badge to the release workflow"
        );
        assert!(
            content.contains(&format!("](https://github.com/{GITHUB_REPOSITORY}/releases)")),
            "{readme} should link the release badge to GitHub releases"
        );
    }
}
