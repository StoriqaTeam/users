macro_rules! validation_errors {
    ({$($field:tt: [$($code:tt -> $value:expr),+]),*}) => {{
        use validator;
        use std::borrow::Cow;
        use std::collections::HashMap;

        let mut errors = validator::ValidationErrors::new();
        $(
            $(
                let error = validator::ValidationError {
                    code: Cow::from($code),
                    message: Some(Cow::from($value)),
                    params: HashMap::new(),
                };

                errors.add($field, error);
            )+
        )*

        errors
    }}
}


#[cfg(test)]
mod tests {
    use std::vec::Vec;
    use serde_json;

    #[test]
    fn several_errors() {
        let errors = validation_errors!({
            "email": ["invalid" -> "Invalid email", "exists" -> "Already exists"],
            "password": ["match" -> "Doesn't match"]
        });
        let json = serde_json::from_str::<serde_json::Value>(&serde_json::to_string(&errors).unwrap()).unwrap();

        assert_eq!(json["email"][0]["code"], "invalid");
        assert_eq!(json["email"][0]["message"], "Invalid email");
        assert_eq!(json["email"][1]["code"], "exists");
        assert_eq!(json["email"][1]["message"], "Already exists");
        assert_eq!(json["password"][0]["code"], "match");
        assert_eq!(json["password"][0]["message"], "Doesn't match");
    }
}
