use sea_orm::entity::prelude::*;

pub mod organization {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "organizations")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub name: String,
        pub legal_name: Option<String>,
        pub default_currency: String,
        pub timezone: String,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod user {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "users")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub full_name: String,
        pub email: String,
        pub password_hash: String,
        pub access_role: String,
        pub is_active: bool,
        pub failed_login_attempts: i32,
        pub locked_until: Option<DateTimeUtc>,
        pub last_login_at: Option<DateTimeUtc>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod property {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "properties")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub code: String,
        pub name: String,
        pub property_type: String,
        pub description: Option<String>,
        pub address_line_1: String,
        pub address_line_2: Option<String>,
        pub city: String,
        pub region: Option<String>,
        pub postal_code: Option<String>,
        pub country_code: String,
        pub status: String,
        pub created_by: Option<String>,
        pub updated_by: Option<String>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
        pub deleted_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod unit {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "property_units")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub property_id: String,
        pub unit_number: String,
        pub unit_type: String,
        pub floor_number: Option<i32>,
        pub area_square_meters: Option<Decimal>,
        pub bedrooms: i32,
        pub bathrooms: Decimal,
        pub market_rent_minor: Option<i64>,
        pub currency_code: Option<String>,
        pub status: String,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
        pub deleted_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod tenant {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "tenants")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub full_name: String,
        pub email: Option<String>,
        pub phone: Option<String>,
        pub mailing_address: Option<String>,
        pub notes: Option<String>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
        pub deleted_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod lease {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "leases")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub lease_number: String,
        pub unit_id: String,
        pub tenant_id: String,
        pub start_on: Date,
        pub end_on: Date,
        pub rent_amount_minor: i64,
        pub deposit_amount_minor: i64,
        pub currency_code: String,
        pub billing_period: String,
        pub due_day: i32,
        pub status: String,
        pub terms: Option<String>,
        pub signed_at: Option<DateTimeUtc>,
        pub terminated_at: Option<DateTimeUtc>,
        pub created_by: Option<String>,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
        pub deleted_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod lease_charge {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "lease_charges")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub lease_id: String,
        pub tenant_id: String,
        pub charge_type: String,
        pub description: Option<String>,
        pub period_start: Date,
        pub period_end: Date,
        pub due_on: Date,
        pub amount_minor: i64,
        pub currency_code: String,
        pub status: String,
        pub created_at: DateTimeUtc,
        pub updated_at: DateTimeUtc,
        pub voided_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod payment {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "payments")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub tenant_id: String,
        pub received_at: DateTimeUtc,
        pub amount_minor: i64,
        pub currency_code: String,
        pub payment_method: String,
        pub external_reference: Option<String>,
        pub status: String,
        pub notes: Option<String>,
        pub recorded_by: Option<String>,
        pub created_at: DateTimeUtc,
        pub voided_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod payment_allocation {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "payment_allocations")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub payment_id: String,
        pub lease_charge_id: String,
        pub tenant_id: String,
        pub currency_code: String,
        pub amount_minor: i64,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod user_session {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "user_sessions")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub user_id: String,
        pub token_hash: String,
        pub created_at: DateTimeUtc,
        pub expires_at: DateTimeUtc,
        pub revoked_at: Option<DateTimeUtc>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod audit_event {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "audit_events")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: String,
        pub organization_id: String,
        pub actor_user_id: Option<String>,
        pub action: String,
        pub entity_type: String,
        pub entity_id: String,
        pub details_json: Option<String>,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
