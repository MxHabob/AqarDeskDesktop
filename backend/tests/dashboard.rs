use aqardesk_backend::{
    application::{DashboardService, StatusCount},
    domain::{AccessRole, ActorContext, OrganizationId, Permission, UserId},
    infrastructure::SeaOrmDashboardRepository,
    migration,
};
use chrono::{Datelike, Months, NaiveDate, TimeZone, Utc};
use chrono_tz::Asia::Riyadh;
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use uuid::Uuid;

#[tokio::test]
async fn dashboard_reports_organization_scoped_live_counts_and_posted_monthly_totals() {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("connect test database");
    migration::initialize(&database)
        .await
        .expect("apply platform migrations");

    let organization_id = Uuid::new_v4().to_string();
    let other_organization_id = Uuid::new_v4().to_string();
    execute(
        &database,
        "INSERT INTO organizations (id, name, default_currency, timezone) VALUES (?, ?, ?, ?)",
        vec![
            organization_id.clone().into(),
            "Dashboard fixture".into(),
            "SAR".into(),
            "Asia/Riyadh".into(),
        ],
    )
    .await;
    execute(
        &database,
        "INSERT INTO organizations (id, name, default_currency, timezone) VALUES (?, ?, ?, ?)",
        vec![
            other_organization_id.clone().into(),
            "Other organization".into(),
            "SAR".into(),
            "Asia/Riyadh".into(),
        ],
    )
    .await;

    let now = Utc::now().with_timezone(&Riyadh).date_naive();
    let current_month = NaiveDate::from_ymd_opt(now.year(), now.month(), 1).expect("current month");
    let previous_month = current_month
        .checked_sub_months(Months::new(1))
        .expect("previous month");
    let current_payment_at = local_midday(current_month);
    let previous_payment_at = local_midday(previous_month);
    let property_id = Uuid::new_v4().to_string();
    let inactive_property_id = Uuid::new_v4().to_string();
    insert_property(&database, &organization_id, &property_id, "active").await;
    insert_property(
        &database,
        &organization_id,
        &inactive_property_id,
        "inactive",
    )
    .await;
    let other_property_id = Uuid::new_v4().to_string();
    insert_property(
        &database,
        &other_organization_id,
        &other_property_id,
        "active",
    )
    .await;

    let unit_id = Uuid::new_v4().to_string();
    insert_unit(
        &database,
        &organization_id,
        &property_id,
        &unit_id,
        "available",
    )
    .await;
    insert_unit(
        &database,
        &organization_id,
        &property_id,
        &Uuid::new_v4().to_string(),
        "maintenance",
    )
    .await;
    insert_unit(
        &database,
        &other_organization_id,
        &other_property_id,
        &Uuid::new_v4().to_string(),
        "available",
    )
    .await;

    let tenant_id = Uuid::new_v4().to_string();
    execute(
        &database,
        "INSERT INTO tenants (id, organization_id, full_name, phone) VALUES (?, ?, ?, ?)",
        vec![
            tenant_id.clone().into(),
            organization_id.clone().into(),
            "Current tenant".into(),
            "5550001".into(),
        ],
    )
    .await;
    let other_tenant_id = Uuid::new_v4().to_string();
    execute(
        &database,
        "INSERT INTO tenants (id, organization_id, full_name, phone) VALUES (?, ?, ?, ?)",
        vec![
            other_tenant_id.clone().into(),
            other_organization_id.clone().into(),
            "Other tenant".into(),
            "5550002".into(),
        ],
    )
    .await;

    insert_payment(
        &database,
        &organization_id,
        &tenant_id,
        current_payment_at,
        "posted",
        None,
        12_500,
    )
    .await;
    insert_payment(
        &database,
        &organization_id,
        &tenant_id,
        current_payment_at,
        "void",
        Some(current_payment_at),
        91_000,
    )
    .await;
    insert_payment(
        &database,
        &organization_id,
        &tenant_id,
        previous_payment_at,
        "posted",
        None,
        78_000,
    )
    .await;
    insert_payment(
        &database,
        &other_organization_id,
        &other_tenant_id,
        current_payment_at,
        "posted",
        None,
        99_000,
    )
    .await;

    let actor = ActorContext::new(
        UserId::new(),
        OrganizationId::parse(&organization_id).expect("organization identifier"),
        AccessRole::Admin,
    );
    let summary = DashboardService::new(SeaOrmDashboardRepository::new(&database))
        .summary(actor, 1)
        .await
        .expect("load dashboard summary");

    assert_eq!(summary.active_properties_count, 1);
    assert_eq!(summary.units_count, 2);
    assert_eq!(summary.available_units_count, 1);
    assert_eq!(summary.tenants_count, 1);
    assert_eq!(summary.posted_payments_count, 2);
    assert_eq!(
        summary.unit_status_counts,
        vec![
            StatusCount {
                status: "available".to_string(),
                count: 1
            },
            StatusCount {
                status: "maintenance".to_string(),
                count: 1
            }
        ]
    );
    assert_eq!(summary.payment_months.len(), 1);
    assert_eq!(
        summary.payment_months[0],
        current_month.format("%Y-%m").to_string()
    );
    assert_eq!(summary.monthly_payments.len(), 1);
    assert_eq!(summary.monthly_payments[0].amount_minor, 12_500);
    assert_eq!(summary.monthly_payments[0].payment_count, 1);
    assert_eq!(summary.payments_by_currency.len(), 1);
    assert_eq!(summary.payments_by_currency[0].amount_minor, 12_500);
    assert_eq!(summary.payments_by_currency[0].payment_count, 1);
    assert_eq!(
        summary.payment_status_counts,
        vec![
            StatusCount {
                status: "posted".to_string(),
                count: 2
            },
            StatusCount {
                status: "void".to_string(),
                count: 1
            }
        ]
    );
}

#[tokio::test]
async fn property_dashboard_reports_scoped_currency_totals_and_respects_payment_permission() {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("connect test database");
    migration::initialize(&database)
        .await
        .expect("apply platform migrations");

    let organization_id = Uuid::new_v4().to_string();
    execute(
        &database,
        "INSERT INTO organizations (id, name, default_currency, timezone) VALUES (?, ?, ?, ?)",
        vec![
            organization_id.clone().into(),
            "Property dashboard fixture".into(),
            "SAR".into(),
            "Asia/Riyadh".into(),
        ],
    )
    .await;

    let property_id = Uuid::new_v4().to_string();
    let other_property_id = Uuid::new_v4().to_string();
    insert_property(&database, &organization_id, &property_id, "active").await;
    insert_property(&database, &organization_id, &other_property_id, "active").await;
    let first_unit_id = Uuid::new_v4().to_string();
    let second_unit_id = Uuid::new_v4().to_string();
    let other_unit_id = Uuid::new_v4().to_string();
    insert_unit(
        &database,
        &organization_id,
        &property_id,
        &first_unit_id,
        "available",
    )
    .await;
    insert_unit(
        &database,
        &organization_id,
        &property_id,
        &second_unit_id,
        "available",
    )
    .await;
    insert_unit(
        &database,
        &organization_id,
        &other_property_id,
        &other_unit_id,
        "available",
    )
    .await;

    let first_tenant_id = Uuid::new_v4().to_string();
    let second_tenant_id = Uuid::new_v4().to_string();
    for (tenant_id, name) in [
        (&first_tenant_id, "First property tenant"),
        (&second_tenant_id, "Second property tenant"),
    ] {
        execute(
            &database,
            "INSERT INTO tenants (id, organization_id, full_name, phone) VALUES (?, ?, ?, ?)",
            vec![
                tenant_id.clone().into(),
                organization_id.clone().into(),
                name.into(),
                "5550100".into(),
            ],
        )
        .await;
    }

    let user_id = Uuid::new_v4().to_string();
    execute(
        &database,
        "INSERT INTO users (id, organization_id, full_name, email, password_hash, access_role) VALUES (?, ?, ?, ?, ?, ?)",
        vec![
            user_id.clone().into(),
            organization_id.clone().into(),
            "Dashboard user".into(),
            format!("{user_id}@example.test").into(),
            "test-hash".into(),
            "admin".into(),
        ],
    )
    .await;
    let template_id = Uuid::new_v4().to_string();
    let template_version_id = Uuid::new_v4().to_string();
    execute(
        &database,
        "INSERT INTO contract_templates (id, organization_id, template_key, name, lease_type, locale, created_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
        vec![
            template_id.clone().into(),
            organization_id.clone().into(),
            "dashboard-fixture".into(),
            "Dashboard fixture".into(),
            "residential".into(),
            "en".into(),
            user_id.clone().into(),
        ],
    )
    .await;
    execute(
        &database,
        "INSERT INTO contract_template_versions (id, organization_id, template_id, version_number, content_format, content_body, placeholders_json, content_sha256, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            template_version_id.clone().into(),
            organization_id.clone().into(),
            template_id.clone().into(),
            1_i32.into(),
            "markdown".into(),
            "Fixture contract".into(),
            "{}".into(),
            "fixture-hash".into(),
            user_id.clone().into(),
        ],
    )
    .await;

    let sar_contract_id = Uuid::new_v4().to_string();
    let usd_contract_id = Uuid::new_v4().to_string();
    let excluded_contract_id = Uuid::new_v4().to_string();
    insert_contract(
        &database,
        &organization_id,
        &sar_contract_id,
        "SAR-1",
        &property_id,
        &first_unit_id,
        &first_tenant_id,
        "SAR",
        100_000,
        &template_id,
        &template_version_id,
        &user_id,
    )
    .await;
    insert_contract(
        &database,
        &organization_id,
        &usd_contract_id,
        "USD-1",
        &property_id,
        &second_unit_id,
        &second_tenant_id,
        "USD",
        50_000,
        &template_id,
        &template_version_id,
        &user_id,
    )
    .await;
    insert_contract(
        &database,
        &organization_id,
        &excluded_contract_id,
        "OTHER-1",
        &other_property_id,
        &other_unit_id,
        &first_tenant_id,
        "SAR",
        900_000,
        &template_id,
        &template_version_id,
        &user_id,
    )
    .await;

    let posted_payment_id = Uuid::new_v4().to_string();
    let void_payment_id = Uuid::new_v4().to_string();
    insert_payment_with_id(
        &database,
        &organization_id,
        &posted_payment_id,
        &first_tenant_id,
        "SAR",
        "posted",
        None,
    )
    .await;
    insert_payment_with_id(
        &database,
        &organization_id,
        &void_payment_id,
        &second_tenant_id,
        "USD",
        "void",
        Some(Utc::now()),
    )
    .await;
    insert_allocation(
        &database,
        &organization_id,
        &posted_payment_id,
        &sar_contract_id,
        40_000,
    )
    .await;
    insert_allocation(
        &database,
        &organization_id,
        &void_payment_id,
        &usd_contract_id,
        25_000,
    )
    .await;

    let actor = ActorContext::new(
        UserId::new(),
        OrganizationId::parse(&organization_id).expect("organization identifier"),
        AccessRole::Admin,
    );
    let service = DashboardService::new(SeaOrmDashboardRepository::new(&database));
    let summary = service
        .property_summary(actor, &property_id)
        .await
        .expect("load property dashboard summary");
    assert_eq!(summary.contract_count, 2);
    assert_eq!(summary.tenant_count, 2);
    assert_eq!(
        summary
            .rent_by_currency
            .iter()
            .map(|total| (total.currency_code.as_str(), total.amount_minor))
            .collect::<Vec<_>>(),
        vec![("SAR", 100_000), ("USD", 50_000)]
    );
    assert_eq!(
        summary
            .posted_allocations_by_currency
            .expect("admin can view payments")
            .iter()
            .map(|total| (total.currency_code.as_str(), total.amount_minor))
            .collect::<Vec<_>>(),
        vec![("SAR", 40_000)]
    );

    let contracts_only_actor = ActorContext::with_permissions(
        UserId::new(),
        OrganizationId::parse(&organization_id).expect("organization identifier"),
        AccessRole::Staff,
        vec![Permission::ViewProperties, Permission::ViewContracts],
    );
    let restricted_summary = service
        .property_summary(contracts_only_actor, &property_id)
        .await
        .expect("load contract-only summary");
    assert!(restricted_summary.posted_allocations_by_currency.is_none());
}

async fn insert_property(
    database: &sea_orm::DatabaseConnection,
    organization_id: &str,
    id: &str,
    status: &str,
) {
    execute(
        database,
        "INSERT INTO properties (id, organization_id, code, name, property_type, address_line_1, city, country_code, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            id.to_string().into(),
            organization_id.to_string().into(),
            format!("P-{}", id[..8].to_ascii_uppercase()).into(),
            "Fixture property".into(),
            "residential".into(),
            "Main road".into(),
            "Riyadh".into(),
            "SA".into(),
            status.to_string().into(),
        ],
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn insert_contract(
    database: &sea_orm::DatabaseConnection,
    organization_id: &str,
    contract_id: &str,
    contract_number: &str,
    property_id: &str,
    unit_id: &str,
    tenant_id: &str,
    currency_code: &str,
    rent_amount_minor: i64,
    template_id: &str,
    template_version_id: &str,
    user_id: &str,
) {
    execute(
        database,
        "INSERT INTO contract_documents (id, organization_id, contract_number, template_id, template_version_id, template_name, template_locale, property_id, property_name, property_code, unit_id, unit_number, tenant_id, tenant_name, start_date, end_date, currency_code, currency_decimal_digits, rent_amount_minor, deposit_amount_minor, document_body, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            contract_id.to_string().into(),
            organization_id.to_string().into(),
            contract_number.to_string().into(),
            template_id.to_string().into(),
            template_version_id.to_string().into(),
            "Fixture template".into(),
            "en".into(),
            property_id.to_string().into(),
            "Fixture property".into(),
            "FIXTURE".into(),
            unit_id.to_string().into(),
            "Unit 1".into(),
            tenant_id.to_string().into(),
            "Fixture tenant".into(),
            "2026-01-01".into(),
            "2026-12-31".into(),
            currency_code.to_string().into(),
            2_i16.into(),
            rent_amount_minor.into(),
            0_i64.into(),
            "Fixture document".into(),
            user_id.to_string().into(),
        ],
    )
    .await;
}

async fn insert_payment_with_id(
    database: &sea_orm::DatabaseConnection,
    organization_id: &str,
    payment_id: &str,
    tenant_id: &str,
    currency_code: &str,
    status: &str,
    voided_at: Option<chrono::DateTime<Utc>>,
) {
    execute(
        database,
        "INSERT INTO payments (id, organization_id, tenant_id, received_at, amount_minor, currency_code, payment_method, status, voided_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            payment_id.to_string().into(),
            organization_id.to_string().into(),
            tenant_id.to_string().into(),
            Utc::now().to_rfc3339().into(),
            50_000_i64.into(),
            currency_code.to_string().into(),
            "cash".into(),
            status.to_string().into(),
            voided_at.map(|date| date.to_rfc3339()).into(),
        ],
    )
    .await;
}

async fn insert_allocation(
    database: &sea_orm::DatabaseConnection,
    organization_id: &str,
    payment_id: &str,
    contract_id: &str,
    amount_minor: i64,
) {
    execute(
        database,
        "INSERT INTO payment_allocations (id, organization_id, payment_id, contract_document_id, contract_number, property_name, property_code, unit_number, amount_minor) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Uuid::new_v4().to_string().into(),
            organization_id.to_string().into(),
            payment_id.to_string().into(),
            contract_id.to_string().into(),
            "Fixture contract".into(),
            "Fixture property".into(),
            "FIXTURE".into(),
            "Unit 1".into(),
            amount_minor.into(),
        ],
    )
    .await;
}

async fn insert_unit(
    database: &sea_orm::DatabaseConnection,
    organization_id: &str,
    property_id: &str,
    id: &str,
    status: &str,
) {
    execute(
        database,
        "INSERT INTO property_units (id, organization_id, property_id, unit_number, unit_type, bedrooms, bathrooms, status) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            id.to_string().into(),
            organization_id.to_string().into(),
            property_id.to_string().into(),
            format!("U-{}", &id[..8]).into(),
            "apartment".into(),
            1_i32.into(),
            1.0_f64.into(),
            status.to_string().into(),
        ],
    )
    .await;
}

async fn insert_payment(
    database: &sea_orm::DatabaseConnection,
    organization_id: &str,
    tenant_id: &str,
    received_at: chrono::DateTime<Utc>,
    status: &str,
    voided_at: Option<chrono::DateTime<Utc>>,
    amount_minor: i64,
) {
    execute(
        database,
        "INSERT INTO payments (id, organization_id, tenant_id, received_at, amount_minor, currency_code, payment_method, status, voided_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        vec![
            Uuid::new_v4().to_string().into(),
            organization_id.to_string().into(),
            tenant_id.to_string().into(),
            received_at.to_rfc3339().into(),
            amount_minor.into(),
            "SAR".into(),
            "cash".into(),
            status.to_string().into(),
            voided_at.map(|date| date.to_rfc3339()).into(),
        ],
    )
    .await;
}

fn local_midday(date: NaiveDate) -> chrono::DateTime<Utc> {
    Riyadh
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 12, 0, 0)
        .single()
        .expect("midday exists")
        .with_timezone(&Utc)
}

async fn execute(database: &sea_orm::DatabaseConnection, query: &str, values: Vec<sea_orm::Value>) {
    database
        .execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            query,
            values,
        ))
        .await
        .expect("execute dashboard fixture statement");
}
