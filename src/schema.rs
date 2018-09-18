table! {
    identities (user_id) {
        user_id -> Int4,
        email -> Varchar,
        password -> Nullable<Varchar>,
        provider -> Varchar,
        saga_id -> Varchar,
    }
}

table! {
    reset_tokens (token) {
        token -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
        token_type -> Varchar,
    }
}

table! {
    user_roles (id) {
        id -> Int4,
        user_id -> Int4,
        role -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        email_verified -> Bool,
        phone -> Nullable<Varchar>,
        phone_verified -> Bool,
        is_active -> Bool,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        middle_name -> Nullable<Varchar>,
        gender -> Nullable<Varchar>,
        birthdate -> Nullable<Date>,
        last_login_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        saga_id -> Varchar,
        avatar -> Nullable<Varchar>,
        is_blocked -> Bool,
    }
}

joinable!(identities -> users (user_id));
joinable!(user_roles -> users (user_id));

allow_tables_to_appear_in_same_query!(identities, reset_tokens, user_roles, users,);
