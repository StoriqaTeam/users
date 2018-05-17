//! Models for managing user delivery address
use validator::Validate;

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

#[derive(Serialize, Queryable, Insertable, Debug, Default)]
#[table_name = "user_delivery_address"]
pub struct UserDeliveryAddress {
    pub id: i32,
    pub user_id: i32,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub country: String,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: String,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub is_priority: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable, Validate)]
#[table_name = "user_delivery_address"]
pub struct NewUserDeliveryAddress {
    pub user_id: i32,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    #[validate(length(min = "1", message = "Country must not be empty"))]
    pub country: String,
    pub locality: Option<String>,
    pub political: Option<String>,
    #[validate(length(min = "1", message = "Postal code must not be empty"))]
    pub postal_code: String,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub is_priority: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Insertable, AsChangeset, Validate)]
#[table_name = "user_delivery_address"]
pub struct UpdateUserDeliveryAddress {
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    #[validate(length(min = "1", message = "Country must not be empty"))]
    pub country: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    #[validate(length(min = "1", message = "Postal code must not be empty"))]
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub address: Option<String>,
    pub is_priority: Option<bool>,
}
