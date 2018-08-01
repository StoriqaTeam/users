table! {
    users (id) {
        id -> Integer,
        email -> Varchar,
        email_verified -> Bool,
        phone -> Nullable<VarChar>,
        phone_verified -> Bool,
        is_active -> Bool ,
        first_name -> Nullable<VarChar>,
        last_name -> Nullable<VarChar>,
        middle_name -> Nullable<VarChar>,
        gender -> Nullable<VarChar>,
        birthdate -> Nullable<Date>,
        avatar -> Nullable<VarChar>,
        last_login_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        saga_id -> VarChar,
    }
}

table! {
    identities (user_id) {
        user_id -> Integer,
        email -> Varchar,
        password -> Nullable<VarChar>,
        provider -> Varchar,
        saga_id -> VarChar,
    }
}

table! {
    reset_tokens (token) {
        token -> VarChar,
        email -> VarChar,
        token_type -> VarChar,
        created_at -> Timestamp,
    }
}

table! {
    user_delivery_address (id) {
        id -> Integer,
        user_id -> Integer,
        administrative_area_level_1 -> Nullable<VarChar>,
        administrative_area_level_2 -> Nullable<VarChar>,
        country -> VarChar,
        locality -> Nullable<VarChar>,
        political -> Nullable<VarChar>,
        postal_code -> VarChar,
        route -> Nullable<VarChar>,
        street_number -> Nullable<VarChar>,
        address -> Nullable<VarChar>,
        is_priority -> Bool,
    }
}

table! {
    user_roles (id) {
        id -> Integer,
        user_id -> Integer,
        role -> VarChar,
    }
}
