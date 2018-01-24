table! {
    identities (user_id) {
        user_id -> Int4,
        user_email -> Varchar,
        user_password -> Nullable<Varchar>,
        provider -> Varchar,
    }
}

table! {
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
        gender -> Varchar,
        birthdate -> Nullable<Timestamp>, // 
        last_login_at -> Timestamp, // 
        created_at -> Timestamp, // UTC 0, generated at db level
        updated_at -> Timestamp, // UTC 0, generated at db level
    }
}