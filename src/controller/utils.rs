use std::collections::HashMap;
use std::iter::FromIterator;

/// Splits query string to key-value pairs. See `macros::parse_query` for more sophisticated parsing.
// TODO: Cover more complex cases, e.g. `from=count=10`
pub fn query_params(query: &str) -> HashMap<&str, &str> {
    HashMap::from_iter(query.split("&").map(|pair| {
        let mut params = pair.split("=");
        (params.next().unwrap(), params.next().unwrap_or(""))
    }))
}
