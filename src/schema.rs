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
        uuid -> Uuid,
        updated_at -> Timestamp,
    }
}

table! {
    user_roles (id) {
        user_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        name -> Varchar,
        data -> Nullable<Jsonb>,
        id -> Uuid,
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
        emarsys_id -> Nullable<Int4>,
        referal -> Nullable<Int4>,
        utm_marks -> Nullable<Jsonb>,
        country -> Nullable<Varchar>,
        referer -> Nullable<Varchar>,
        revoke_before -> Timestamp,
    }
}

joinable!(identities -> users (user_id));
joinable!(user_roles -> users (user_id));

allow_tables_to_appear_in_same_query!(
    identities,
    reset_tokens,
    user_roles,
    users,
);
