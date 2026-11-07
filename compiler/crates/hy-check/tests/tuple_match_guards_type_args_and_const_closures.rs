//! Regression tests for T23 (`HYB-E2002`) across `option.some(b)` and `enum`
//! match guards, generic type arguments / `make(chan [pair()]int, 1)`, and
//! multi-value local / `for ... in` / `match` bindings in `const` closures.

use hy_check::{check_file, Verdict};
use hy_syntax::parse_file;

fn check_hy(src: &str) -> hy_check::CheckResult {
    let ast = parse_file("test.hy", src).expect("source must parse");
    check_file(&ast)
}

fn assert_single_e2002(src: &str) {
    let r = check_hy(src);
    assert_eq!(r.verdict, Verdict::Reject, "expected Reject for:\n{src}\ngot: {:?}", r.diagnostics);
    let e2002: Vec<_> = r.diagnostics.iter().filter(|d| d.code == "HYB-E2002").collect();
    assert_eq!(e2002.len(), 1, "expected 1 HYB-E2002, got {} in {:?}", e2002.len(), r.diagnostics);
}

#[test]
fn match_guard_qualified_option_some_and_enum_variant_payload_rejected() {
    assert_single_e2002(
        "fn pair() (int, int) { return 1, 2 }\n\
         fn main() {\n\
             opt := some(pair)\n\
             match opt { option.some(f) if f() == 1: {}\n some(_): {}\n none: {} }\n\
         }\n",
    );
    assert_single_e2002(
        "enum pkt { cb(fn() int int)\n empty }\n\
         fn single() int { return 1 }\n\
         fn get_pkt() pkt { return pkt.empty }\n\
         fn main() {\n\
             f := single\n\
             p := get_pkt()\n\
             match p { pkt.cb(f) if f() == 1: {}\n pkt.cb(_): {}\n pkt.empty: {} }\n\
             _ = f\n\
         }\n",
    );
}

#[test]
fn generic_type_arg_and_make_chan_array_len_multi_value_call_rejected() {
    assert_single_e2002(
        "struct box[t] { v t }\n\
         fn pair() (int, int) { return 1, 2 }\n\
         fn main() { var b box[[pair()]int]\n _ = b }\n",
    );
    assert_single_e2002(
        "fn pair() (int, int) { return 1, 2 }\n\
         fn main() { ch := make(chan [pair()]int, 1)\n _ = ch }\n",
    );
}

#[test]
fn const_closure_multi_value_local_for_iterate_and_match_some_rejected() {
    assert_single_e2002(
        "fn get_cb_pair() (fn() int int, bool) { return fn() (int, int) { return 1, 2 }, true }\n\
         const fns = []fn(){fn() { cb, ok := get_cb_pair()\n _ = ok\n x := cb()\n _ = x }}\n\
         fn main() { _ = fns }\n",
    );
    assert_single_e2002(
        "const fns = []fn(items []fn() int int){fn(items []fn() int int) {\n\
             for cb in items { x := cb()\n _ = x }\n\
         }}\n\
         fn main() { _ = fns }\n",
    );
    assert_single_e2002(
        "const fns = []fn(opt option[fn() int int]){fn(opt option[fn() int int]) {\n\
             match opt { some(cb): { x := cb()\n _ = x }\n none: {} }\n\
         }}\n\
         fn main() { _ = fns }\n",
    );
}

#[test]
fn valid_match_guard_and_const_closure_single_value_accepted() {
    let r = check_hy(
        "enum pkt { cb(fn() int)\n empty }\n\
         fn single() int { return 7 }\n\
         fn get_pkt() pkt { return pkt.empty }\n\
         const fns = []fn(items []fn() int){fn(items []fn() int) {\n\
             for cb in items { x := cb()\n _ = x }\n\
         }}\n\
         fn main() {\n\
             p := get_pkt()\n\
             match p { pkt.cb(f) if f() == 7: {}\n pkt.cb(_): {}\n pkt.empty: {} }\n\
             _ = fns\n _ = single\n\
         }\n",
    );
    assert_eq!(r.verdict, Verdict::Accept, "expected Accept, got: {:?}", r.diagnostics);
}
