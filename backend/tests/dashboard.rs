use aqardesk_backend::{
    application::{DashboardService, StatusCount},
    domain::{AccessRole, ActorContext, OrganizationId, UserId},
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
