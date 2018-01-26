table! {
    users (id) {
        id -> Integer,
        email -> VarChar,
        password -> VarChar,
        is_active -> Bool,
    }
}

table! {
    user_roles (id) {
        id -> Integer,
        user_id -> Integer,
        role -> VarChar,
    }
}
