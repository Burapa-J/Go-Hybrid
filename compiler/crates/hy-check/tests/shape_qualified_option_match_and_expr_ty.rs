//! Regression tests for `HYB-E5001` match exhaustiveness and `ty_of_expr`
//! across qualified `option.some`/`option.none` patterns, payload-carrying
//! `enum` variant constructor calls (`pkt.cb(1)`, `cb(1)`), and channel
//! receive (`<-ch`) / slice (`s[lo:hi]`) expressions.

use hy_check::{check_file, Verdict};
use hy_syntax::parse_file;

fn check_hy(src: &str) -> hy_check::CheckResult {
    let ast = parse_file("test.hy", src).expect("source must parse");
    check_file(&ast)
}

fn assert_single_e5001(src: &str) {
    let r = check_hy(src);
    assert_eq!(r.verdict, Verdict::Reject, "expected Reject for:\n{src}\ngot: {:?}", r.diagnostics);
    let e5001: Vec<_> = r.diagnostics.iter().filter(|d| d.code == "HYB-E5001").collect();
    assert_eq!(e5001.len(), 1, "expected 1 HYB-E5001, got {} in {:?}", e5001.len(), r.diagnostics);
}

#[test]
fn non_option_subject_with_qualified_option_some_or_none_rejected_e5001() {
    assert_single_e5001(
        "fn main() {\n\
             x := 42\n\
             match x { option.some(v): { _ = v }\n _: {} }\n\
         }\n",
    );
    assert_single_e5001(
        "fn main() {\n\
             x := 42\n\
             match x { option.none: {}\n _: {} }\n\
         }\n",
    );
}

#[test]
fn payload_enum_ctor_call_subject_rejects_option_patterns_and_missing_variants_e5001() {
    assert_single_e5001(
        "enum pkt { cb(int)\n empty }\n\
         fn main() {\n\
             p := pkt.cb(1)\n\
             match p { some(v): { _ = v }\n none: {} }\n\
         }\n",
    );
    assert_single_e5001(
        "enum pkt { cb(int)\n empty }\n\
         fn main() {\n\
             p := cb(1)\n\
             match p { pkt.cb(v): { _ = v } }\n\
         }\n",
    );
    assert_single_e5001(
        "enum pkt { cb(int)\n empty }\n\
         fn main() {\n\
             p := pkt.cb(1)\n\
             match p { option.some(v): { _ = v }\n _: {} }\n\
         }\n",
    );
}

#[test]
fn channel_receive_and_qualified_option_exhaustive_matches_enforced() {
    assert_single_e5001(
        "enum status { pending\n active }\n\
         fn main() {\n\
             ch := make(chan status, 1)\n\
             s := <-ch\n\
             match s { some(v): { _ = v }\n none: {} }\n\
         }\n",
    );
    assert_single_e5001(
        "enum status { pending\n active }\n\
         fn main() {\n\
             arr := [2]status{status.pending, status.active}\n\
             sub := arr[0:1]\n\
             match sub[0] { status.pending: {} }\n\
         }\n",
    );
    let r = check_hy(
        "enum status { pending\n active }\n\
         fn main() {\n\
             ch_opt := make(chan option[int], 1)\n\
             match <-ch_opt { option.some(v): { _ = v }\n option.none: {} }\n\
             ch_st := make(chan status, 1)\n\
             match <-ch_st { status.pending: {}\n status.active: {} }\n\
             o := some(7)\n\
             match o { option.some(v): { _ = v }\n option.none: {} }\n\
         }\n",
    );
    assert_eq!(r.verdict, Verdict::Accept, "expected Accept, got: {:?}", r.diagnostics);
}
