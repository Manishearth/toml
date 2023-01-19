#[test]
fn incomplete_inline_table_issue_296() {
    let err = "native = {".parse::<toml_edit::Document>().unwrap_err();
    snapbox::assert_eq(
        r#"TOML parse error at line 1, column 11
  |
1 | native = {
  |           ^
invalid inline table
expected `}`
"#,
        err.to_string(),
    );
}

#[test]
fn bare_value_disallowed_issue_293() {
    let err = "value=zzz".parse::<toml_edit::Document>().unwrap_err();
    snapbox::assert_eq(
        r#"TOML parse error at line 1, column 7
  |
1 | value=zzz
  |       ^
invalid string
expected `"`, `'`
"#,
        err.to_string(),
    );
}

#[test]
fn bare_value_in_array_disallowed_issue_293() {
    let err = "value=[zzz]".parse::<toml_edit::Document>().unwrap_err();
    snapbox::assert_eq(
        r#"TOML parse error at line 1, column 8
  |
1 | value=[zzz]
  |        ^
invalid array
expected `]`
"#,
        err.to_string(),
    );
}