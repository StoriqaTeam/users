macro_rules! validation_errors {
    ($($field:tt => ($($code:tt -> $value:expr),+)),*) => {{
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

    #[test]
    fn several_errors() {
        let errors = validation_errors!(
            "email" => ("invalid" -> "Invalid email", "exists" -> "Already exists"),
            "password" => ("match" -> "Doesn't match")
        );
        let errors = errors.inner();
        let error = errors.get("email").unwrap().nth(0);
        assert_eq!(error.code.into_owned(), "invalid");
        // assert_eq!(&errors.get("email").unwrap()[0].message.unwrap().into_owned(), "invalid");
    }
}
