macro_rules! get_and_parse {
    ($hash:expr, $t: ty, $key: tt) => ($hash.get($key).and_then(|value| value.parse::<$t>().ok()))
}

#[macro_export]
macro_rules! parse_params {
    ($query: expr, $e:tt -> $t:ty) => ({ let hash = $crate::utils::http::query_params($query); get_and_parse!(hash, $t, $e) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2)) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty, $e3:tt -> $t3:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2), get_and_parse!(hash, $t3, $e3)) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty, $e3:tt -> $t3:ty, $e4:tt -> $t4:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2), get_and_parse!(hash, $t3, $e3), get_and_parse!(hash, $t4, $e4)) });
    ($query: expr, $e1:tt -> $t1:ty, $e2:tt -> $t2:ty, $e3:tt -> $t3:ty, $e4:tt -> $t4:ty, $e5:tt -> $t5:ty) => ({ let hash = $crate::utils::http::query_params($query); (get_and_parse!(hash, $t1, $e1), get_and_parse!(hash, $t2, $e2), get_and_parse!(hash, $t3, $e3), get_and_parse!(hash, $t4, $e4), get_and_parse!(hash, $t5, $e5)) });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn params_1() {
        assert_eq!(parse_params!("from=12", "from" -> i32), Some(12));
        assert_eq!(parse_params!("from=12a", "from" -> i32), None);
        assert_eq!(parse_params!("from=12", "to" -> i32), None);
    }

    #[test]
    fn params_2() {
        assert_eq!(parse_params!("from=12&to=22", "from" -> i32, "to" -> i64), (Some(12), Some(22)));
        assert_eq!(parse_params!("from=12&to=22", "from" -> i32, "to" -> String), (Some(12), Some("22".to_string())));
        assert_eq!(parse_params!("from=12&to=true", "from" -> bool, "to" -> bool), (None, Some(true)));
    }

    #[test]
    fn params_3() {
        assert_eq!(parse_params!("from=12&to=22&published=true", "from" -> i32, "to" -> i64, "published" -> bool), (Some(12), Some(22), Some(true)));
    }

    #[test]
    fn params_4() {
        assert_eq!(parse_params!("from=12&to=22&published=true&name=Alex", "from" -> i32, "to" -> i64, "published" -> bool, "name" -> String), (Some(12), Some(22), Some(true), Some("Alex".to_string())));
    }

    #[test]
    fn params_5() {
        assert_eq!(parse_params!("from=12&to=22&published=true&name=Alex&price=3.25", "from" -> i32, "to" -> i64, "published" -> bool, "name" -> String, "price" -> f32), (Some(12), Some(22), Some(true), Some("Alex".to_string()), Some(3.25)));
    }
}
