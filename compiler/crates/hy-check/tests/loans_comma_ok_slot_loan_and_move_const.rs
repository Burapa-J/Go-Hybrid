use hy_check::{check_file, Verdict};

fn check_codes(src: &str) -> Vec<String> {
    let ast = hy_syntax::parse_file("test.hy", src).expect("test source must parse");
    check_file(&ast).diagnostics.into_iter().map(|d| d.code).collect()
}

fn check(src: &str) -> hy_check::CheckResult {
    let ast = hy_syntax::parse_file("test.hy", src).expect("test source must parse");
    check_file(&ast)
}

#[test]
fn comma_ok_map_slot_loan_on_const_rejects_with_e3006() {
    let src_define = "\
struct svc { count int }
const c_map = map[string]svc{\"a\": svc{count: 1}}
fn main() {
    s, ok := &c_map[\"a\"]
    if ok { _ = s.count }
}
";
    let codes_def = check_codes(src_define);
    assert!(codes_def.iter().any(|c| c == "HYB-E3006"), "got {:?}", codes_def);

    let src_assign = "\
struct svc { count int }
const c_map = map[string]svc{\"a\": svc{count: 1}}
fn main() {
    var s svc = svc{count: 0}
    var ok bool = false
    s, ok = &c_map[\"a\"]
    if ok { _ = s.count }
}
";
    let codes_asgn = check_codes(src_assign);
    assert!(codes_asgn.iter().any(|c| c == "HYB-E3006"), "got {:?}", codes_asgn);
}

#[test]
fn comma_ok_map_slot_loan_enforces_a3_insert_and_o8_uniqueness() {
    let src_insert = "\
struct svc { count int }
fn main() {
    m := map[string]svc{\"a\": svc{count: 1}}
    s, ok := &m[\"a\"]
    m[\"b\"] = svc{count: 2}
    if ok { s.count = 99 }
}
";
    let codes_insert = check_codes(src_insert);
    assert!(codes_insert.iter().any(|c| c == "HYB-E3102"), "got {:?}", codes_insert);

    let src_overlap = "\
struct svc { count int }
fn main() {
    m := map[string]svc{\"a\": svc{count: 1}}
    var s1 svc = svc{count: 0}
    var ok1 bool = false
    s1, ok1 = &m[\"a\"]
    s2, ok2 := &m[\"a\"]
    if ok1 && ok2 {
        s1.count = 10
        s2.count = 20
    }
}
";
    let codes_overlap = check_codes(src_overlap);
    assert!(codes_overlap.iter().any(|c| c == "HYB-E3102"), "got {:?}", codes_overlap);
}

#[test]
fn move_wrapped_const_field_and_index_loans_reject_with_e3006() {
    let src_outer_move = "\
struct inner { v int }
struct cfg { s inner }
const global_cfg = cfg{s: inner{v: 1}}
fn main() {
    p := move &global_cfg.s
    _ = p.v
}
";
    let codes_outer = check_codes(src_outer_move);
    assert!(codes_outer.iter().any(|c| c == "HYB-E3006"), "got {:?}", codes_outer);

    let src_inner_move = "\
struct inner { v int }
struct cfg { s inner }
const global_cfg = cfg{s: inner{v: 1}}
fn main() {
    p := &(move global_cfg).s
    _ = p.v
}
";
    let codes_inner = check_codes(src_inner_move);
    assert!(codes_inner.iter().any(|c| c == "HYB-E3006"), "got {:?}", codes_inner);

    let src_call_move_addr = "\
struct cfg { count int }
const global_cfg = cfg{count: 1}
fn bump(p *cfg) { p.count = 99 }
fn main() { bump(move &global_cfg) }
";
    let codes_call = check_codes(src_call_move_addr);
    assert!(codes_call.iter().any(|c| c == "HYB-E3006"), "got {:?}", codes_call);
}

#[test]
fn valid_mutable_struct_field_loan_still_accepts() {
    let src = "\
struct inner { v int }
struct cfg { s inner }
fn main() {
    c := cfg{s: inner{v: 1}}
    p := &c.s
    p.v = 2
}
";
    let res = check(src);
    assert_eq!(res.verdict, Verdict::Accept, "{:?}", res.diagnostics);
}
