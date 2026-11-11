use aqardesk_backend::{
    application::{PaymentLedger, TenantDirectory, UnitCatalog},
    domain::{
        AccessRole, ActorContext, CurrencyCode, OrganizationId, PaymentInput, PaymentMethod,
        PropertyId, PropertyUnitInput, TenantInput, UnitStatus, UnitType, UserId,
    },
    infrastructure::{SeaOrmPaymentRepository, SeaOrmTenantRepository, SeaOrmUnitRepository},
    migration,
};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

async fn database() -> sea_orm::DatabaseConnection {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("connect SQLite test database");
    for sql in [
        "CREATE TABLE audit_events (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, actor_user_id TEXT, action TEXT NOT NULL, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, details_json TEXT, created_at TEXT NOT NULL)",
        "CREATE TABLE tenants (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, full_name TEXT NOT NULL, email TEXT, phone TEXT, mailing_address TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE property_units (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, property_id TEXT NOT NULL, unit_number TEXT NOT NULL, unit_type TEXT NOT NULL, floor_number INTEGER, area_square_meters REAL, bedrooms INTEGER NOT NULL, bathrooms REAL NOT NULL, market_rent_minor BIGINT, currency_code TEXT, status TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE payments (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, tenant_id TEXT NOT NULL, received_at TEXT NOT NULL, amount_minor BIGINT NOT NULL, currency_code TEXT NOT NULL, payment_method TEXT NOT NULL, external_reference TEXT, status TEXT NOT NULL, notes TEXT, recorded_by TEXT, created_at TEXT NOT NULL, voided_at TEXT)",
    ] {
        database
            .execute(Statement::from_string(DbBackend::Sqlite, sql))
            .await
            .expect("create persistence fixture");
    }
    database
}

fn actor(organization_id: OrganizationId) -> ActorContext {
    ActorContext::new(UserId::new(), organization_id, AccessRole::Admin)
}

#[tokio::test]
async fn tenant_directory_scopes_updates_and_preserves_archived_rows() {
    let database = database().await;
    let organization_id = OrganizationId::new();
    let other_actor = actor(OrganizationId::new());
    let tenant = TenantDirectory::new(SeaOrmTenantRepository::new(&database))
        .create(
            actor(organization_id),
            TenantInput {
                full_name: "Tenant One".to_string(),
                email: Some("tenant@example.com".to_string()),
                phone: None,
                mailing_address: None,
                notes: None,
            },
        )
        .await
        .expect("create tenant");
    let directory = TenantDirectory::new(SeaOrmTenantRepository::new(&database));
    assert_eq!(
        directory
            .list(actor(organization_id))
            .await
            .expect("list")
            .len(),
        1
    );
    assert!(
        directory
            .list(other_actor)
            .await
            .expect("other org list")
            .is_empty()
    );
    directory
        .archive(actor(organization_id), tenant.id)
        .await
        .expect("archive tenant");
    assert!(
        directory
            .list(actor(organization_id))
            .await
            .expect("list")
            .is_empty()
    );
    assert_eq!(
        directory
            .names(actor(organization_id), &[tenant.id])
            .await
            .expect("load archived tenant display name")
            .get(&tenant.id)
            .map(String::as_str),
        Some("Tenant One")
    );
    assert!(
        directory
            .names(other_actor, &[tenant.id])
            .await
            .expect("other organization cannot resolve tenant")
            .is_empty()
    );
}

#[tokio::test]
async fn unit_catalog_validates_inputs_and_scopes_all_operations() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("aqardesk_backend=debug")
        .with_test_writer()
        .try_init();
    let database = database().await;
    let organization_id = OrganizationId::new();
    let catalog = UnitCatalog::new(SeaOrmUnitRepository::new(&database));
    let unit = catalog
        .create(
            actor(organization_id),
            PropertyUnitInput {
                property_id: PropertyId::new(),
                unit_number: "A-1".to_string(),
                unit_type: UnitType::Apartment,
                floor_number: Some(1),
                area_square_meters: Some(Decimal::new(850, 1)),
                bedrooms: 2,
                bathrooms: Decimal::ONE,
                market_rent_minor: None,
                currency_code: None,
                status: UnitStatus::Available,
            },
        )
        .await
        .expect("create unit");
    assert_eq!(
        catalog
            .list(actor(organization_id), Some(unit.input.property_id))
            .await
            .expect("list units")[0]
            .input
            .unit_number,
        "A-1"
    );
    catalog
        .archive(actor(organization_id), unit.id)
        .await
        .expect("archive unit");
    assert!(
        catalog
            .list(actor(organization_id), None)
            .await
            .expect("list archived unit")
            .is_empty()
    );
}

#[tokio::test]
async fn payment_ledger_records_and_voids_with_atomic_audit_events() {
    let database = database().await;
    let organization_id = OrganizationId::new();
    let actor = actor(organization_id);
    let ledger = PaymentLedger::new(SeaOrmPaymentRepository::new(&database));
    let payment = ledger
        .record(
            actor,
            PaymentInput {
                tenant_id: aqardesk_backend::domain::TenantId::new(),
                received_at: Utc::now(),
                amount_minor: 12_500,
                currency_code: CurrencyCode::try_from("sar".to_string()).expect("currency code"),
                payment_method: PaymentMethod::BankTransfer,
                external_reference: Some(" REF-1 ".to_string()),
                notes: None,
            },
        )
        .await
        .expect("record payment");
    assert_eq!(payment.external_reference.as_deref(), Some("REF-1"));

    ledger.void(actor, payment.id).await.expect("void payment");
    assert!(
        ledger
            .list(actor)
            .await
            .expect("list payments")
            .iter()
            .any(|item| item.id == payment.id
                && item.status == aqardesk_backend::domain::PaymentStatus::Void)
    );

    let audit_rows = database
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT COUNT(*) AS count FROM audit_events",
        ))
        .await
        .expect("count audit rows")
        .expect("audit count row")
        .try_get_by::<i64, _>("count")
        .expect("read audit count");
    assert_eq!(audit_rows, 2);
}

#[tokio::test]
async fn migration_preserves_legacy_numeric_unit_measurements() {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("connect SQLite test database");
    for sql in [
        "CREATE TABLE currencies (code TEXT PRIMARY KEY, name TEXT NOT NULL, name_ar TEXT, symbol TEXT NOT NULL, symbol_position TEXT NOT NULL DEFAULT 'prefix', decimal_digits SMALLINT NOT NULL DEFAULT 2, is_active BOOLEAN NOT NULL DEFAULT TRUE, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)",
        "CREATE TABLE organizations (id TEXT PRIMARY KEY, name TEXT NOT NULL, legal_name TEXT, default_currency TEXT NOT NULL, timezone TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)",
        "CREATE TABLE users (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, full_name TEXT NOT NULL, email TEXT NOT NULL, password_hash TEXT NOT NULL, access_role TEXT NOT NULL, is_active BOOLEAN NOT NULL, failed_login_attempts INTEGER NOT NULL, locked_until TEXT, last_login_at TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, UNIQUE (id, organization_id))",
        "CREATE TABLE properties (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, code TEXT NOT NULL, name TEXT NOT NULL, property_type TEXT NOT NULL, description TEXT, address_line_1 TEXT NOT NULL, address_line_2 TEXT, city TEXT NOT NULL, region TEXT, postal_code TEXT, country_code TEXT NOT NULL, status TEXT NOT NULL, created_by TEXT, updated_by TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE property_units (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, property_id TEXT NOT NULL, unit_number TEXT NOT NULL, unit_type TEXT NOT NULL, floor_number INTEGER, area_square_meters NUMERIC(10, 2), bedrooms INTEGER NOT NULL, bathrooms NUMERIC(4, 1) NOT NULL, market_rent_minor BIGINT, currency_code TEXT, status TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE tenants (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, full_name TEXT NOT NULL, email TEXT, phone TEXT, mailing_address TEXT, notes TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, deleted_at TEXT)",
        "CREATE TABLE payments (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, tenant_id TEXT NOT NULL, received_at TEXT NOT NULL, amount_minor BIGINT NOT NULL, currency_code TEXT NOT NULL, payment_method TEXT NOT NULL, external_reference TEXT, status TEXT NOT NULL, notes TEXT, recorded_by TEXT, created_at TEXT NOT NULL, voided_at TEXT)",
        "CREATE TABLE audit_events (id TEXT PRIMARY KEY, organization_id TEXT NOT NULL, actor_user_id TEXT, action TEXT NOT NULL, entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, details_json TEXT, created_at TEXT NOT NULL)",
    ] {
        database
            .execute(Statement::from_string(DbBackend::Sqlite, sql))
            .await
            .expect("create legacy schema fixture");
    }

    let organization_id = OrganizationId::new();
    let unit_id = aqardesk_backend::domain::UnitId::new();
    database
        .execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT INTO property_units (id, organization_id, property_id, unit_number, unit_type, area_square_meters, bedrooms, bathrooms, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            [
                unit_id.to_string().into(),
                organization_id.to_string().into(),
                PropertyId::new().to_string().into(),
                "A-legacy".into(),
                "apartment".into(),
                85.0.into(),
                2.into(),
                1.0.into(),
                "available".into(),
            ],
        ))
        .await
        .expect("insert legacy measurement");

    migration::initialize(&database)
        .await
        .expect("migrate existing database without dropping tables");
    let units = UnitCatalog::new(SeaOrmUnitRepository::new(&database))
        .list(actor(organization_id), None)
        .await
        .expect("read existing unit after migration");
    assert_eq!(units[0].id, unit_id);
    assert_eq!(units[0].input.area_square_meters, Some(Decimal::from(85)));
    assert_eq!(units[0].input.bathrooms, Decimal::ONE);
    let mut updated_input = units[0].input.clone();
    updated_input.area_square_meters = Some(Decimal::new(905, 1));
    updated_input.bathrooms = Decimal::new(15, 1);
    let updated = UnitCatalog::new(SeaOrmUnitRepository::new(&database))
        .update(actor(organization_id), unit_id, updated_input)
        .await
        .expect("update existing legacy unit");
    assert_eq!(updated.input.area_square_meters, Some(Decimal::new(905, 1)));
    assert_eq!(updated.input.bathrooms, Decimal::new(15, 1));
    let created = UnitCatalog::new(SeaOrmUnitRepository::new(&database))
        .create(
            actor(organization_id),
            PropertyUnitInput {
                property_id: PropertyId::new(),
                unit_number: "A-new".to_string(),
                unit_type: UnitType::Apartment,
                floor_number: None,
                area_square_meters: Some(Decimal::new(725, 1)),
                bedrooms: 1,
                bathrooms: Decimal::new(15, 1),
                market_rent_minor: None,
                currency_code: None,
                status: UnitStatus::Available,
            },
        )
        .await
        .expect("insert unit into legacy NUMERIC schema");
    assert_eq!(created.input.area_square_meters, Some(Decimal::new(725, 1)));
}
