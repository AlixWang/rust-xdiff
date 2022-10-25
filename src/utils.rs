use anyhow::Result;
use console::Style;
use similar::{ChangeTag, TextDiff};

pub fn diff_text(text1: &str, text2: &str) -> Result<String> {
    let diff = TextDiff::from_lines(text1, text2);

    for op in diff.ops().to_vec() {
        for change in diff.iter_changes(&op) {
            let (sign, style) = match change.tag() {
                ChangeTag::Delete => ("-", Style::new().red()),
                ChangeTag::Insert => ("+", Style::new().green()),
                ChangeTag::Equal => (" ", Style::new()),
            };
            print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
        }
    }
    todo!()
}
