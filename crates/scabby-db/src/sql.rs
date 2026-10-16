//! The one place a dynamically assembled query string is declared safe.

/// Wrap a query built from this crate's own SQL fragments.
///
/// sqlx accepts a `&'static str` without comment and requires an explicit audit for anything else,
/// because `format!` around a query is how injection gets in. The only strings that go through here
/// are a fixed column projection plus a literal `where` clause; every caller-supplied value travels
/// as a bind parameter, so no user data is ever concatenated into the text.
pub(crate) fn fragment(query: String) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(query)
}
