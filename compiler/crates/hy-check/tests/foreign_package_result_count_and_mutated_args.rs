use hy_check::check_file;
use hy_syntax::parse_file;

fn codes(src: &str) -> Vec<String> {
    let file = parse_file("test.hy", src).expect("parse ok");
    check_file(&file)
        .diagnostics
        .into_iter()
        .map(|d| d.code)
        .collect()
}

#[test]
fn test_package_result_count_rejects_single_bind_on_json_url_os_hex() {
    // `json.marshal` / `json.marshal_indent` return `([]byte, error)` in Go;
    // binding to 1 variable without `?` must emit HYB-E2002 (T23).
    let src_json = r#"
import "encoding/json"

fn bad_json() {
    data := json.marshal("hello")
    _ = data
}
"#;
    assert_eq!(codes(src_json), vec!["HYB-E2002"]);

    // `url.parse` / `url.parse_request_uri` / `url.query_unescape` return `(T, error)`
    // in Go; binding to 1 variable without `?` must emit HYB-E2002.
    let src_url = r#"
import "net/url"

fn bad_url() {
    u := url.parse("https://example.com")
    _ = u
}
"#;
    assert_eq!(codes(src_url), vec!["HYB-E2002"]);

    // `os.open_file` and `hex.decode` return 2 values in Go; single-bind must emit HYB-E2002.
    let src_os_hex = r#"
import "os"
import "encoding/hex"

fn bad_os_hex(dst []byte, src []byte) {
    f := os.open_file("out.txt", 0, 0)
    n := hex.decode(dst, src)
    _ = f
    _ = n
}
"#;
    assert_eq!(codes(src_os_hex), vec!["HYB-E2002", "HYB-E2002"]);
}

#[test]
fn test_package_mutated_arg_rejects_const_destination_in_hex_and_json() {
    // `hex.encode(dst, src)` and `hex.decode(dst, src)` write into `dst` (arg 0)
    // in place; passing a `const` slice root as `dst` must emit HYB-E3006 (T22).
    let src_hex = r#"
import "encoding/hex"

const dst_buf = []byte{0, 0, 0, 0}

fn bad_hex_const(src []byte) {
    _ = hex.encode(dst_buf, src)
}
"#;
    assert_eq!(codes(src_hex), vec!["HYB-E3006"]);

    // `json.unmarshal(data, &target)` mutates `target` (arg 1) in place;
    // passing `&const_binding` or a `const` slice/map root must emit HYB-E3006.
    let src_json = r#"
import "encoding/json"

const cfg_map = map[string]int{"a": 1}

fn bad_json_const(data []byte) {
    _ = json.unmarshal(data, cfg_map)
}
"#;
    assert_eq!(codes(src_json), vec!["HYB-E3006"]);
}

#[test]
fn test_positive_two_var_bind_and_mutable_dst_accepted() {
    let src_ok = r#"
import "encoding/json"
import "encoding/hex"
import "net/url"

fn ok(src []byte) {
    data, err1 := json.marshal("hello")
    u, err2 := url.parse("https://example.com")
    dst := []byte{0, 0, 0, 0}
    n, err3 := hex.decode(dst, src)
    _ = data
    _ = err1
    _ = u
    _ = err2
    _ = n
    _ = err3
}
"#;
    assert!(codes(src_ok).is_empty());
}
