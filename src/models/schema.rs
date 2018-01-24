table! {
    use diesel::sql_types::*;
    use models::user::ProviderType;
    identities (user_id) {
        user_id -> Int4,
        user_email -> Varchar,
        user_password -> Nullable<Varchar>,
        provider -> ProviderType,
    }
}

table! {
    use diesel::sql_types::*;
    use models::user::GenderType;
    users (id) {
        id -> Int4,
        email -> Varchar,
        email_verified -> Bool,
        phone -> Nullable<VarChar>,
        phone_verified -> Bool,
        is_active -> Bool ,
        first_name -> Nullable<VarChar>,
        last_name -> Nullable<VarChar>,
        middle_name -> Nullable<VarChar>,
        gender -> GenderType,
        birthdate -> Nullable<Timestamp>, // 
        last_login_at -> Timestamp, // 
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}