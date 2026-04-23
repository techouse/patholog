pub(super) fn finish_lines(lines: impl IntoIterator<Item = String>) -> String {
    lines.into_iter().collect::<Vec<_>>().join("\n") + "\n"
}
