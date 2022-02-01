use crate::Query;

#[test]
fn query_debug() {
    assert_eq!(format!("{:?}", Query::new_empty()),"Query { data: [] }");
}