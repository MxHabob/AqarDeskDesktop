use aqardesk_backend::{
    application::{
        AuditLog, ContractTemplateInput, ContractTemplateService, CurrencyCatalog,
        OrganizationSettingsInput, OrganizationSettingsService, ProfileService,
        ProfileSettingsInput,
    },
    domain::{AccessRole, ActorContext, OrganizationId, UserId},
    infrastructure::{
        SeaOrmAuditLogRepository, SeaOrmContractTemplateRepository, SeaOrmCurrencyRepository,
        SeaOrmOrganizationSettingsRepository, SeaOrmProfileRepository,
    },
    migration,
};
use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use uuid::Uuid;

#[tokio::test]
async fn administration_services_persist_version_audit_and_profile_changes() {
    let database = Database::connect("sqlite::memory:")
        .await
        .expect("connect test database");
    migration::initialize(&database)
        .await
        .expect("apply platform migrations");

    let organization_id = Uuid::new_v4().to_string();
    let user_id = Uuid::new_v4().to_string();
    database
        .execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT INTO organizations (id, name, default_currency, timezone) VALUES (?, ?, ?, ?)",
            vec![
                organization_id.clone().into(),
                "Initial name".into(),
                "SAR".into(),
                "Asia/Riyadh".into(),
            ],
        ))
        .await
        .expect("insert organization");
    database
        .execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT INTO users (id, organization_id, full_name, email, password_hash, access_role) VALUES (?, ?, ?, ?, ?, ?)",
            vec![
                user_id.clone().into(),
                organization_id.clone().into(),
                "Admin User".into(),
                "admin@example.test".into(),
                "not-used".into(),
                "admin".into(),
            ],
        ))
        .await
        .expect("insert user");
    database
        .execute(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            "INSERT INTO organization_currencies (organization_id, currency_code, is_enabled, created_by) VALUES (?, ?, ?, ?)",
            vec![
                organization_id.clone().into(),
                "SAR".into(),
                true.into(),
                user_id.clone().into(),
            ],
        ))
        .await
        .expect("enable default currency");

    let organization = OrganizationId::parse(&organization_id).expect("organization identifier");
    let user = UserId::parse(&user_id).expect("user identifier");
    let actor = ActorContext::new(user, organization, AccessRole::Admin);
    let currencies = CurrencyCatalog::new(SeaOrmCurrencyRepository::new(&database));
    assert!(
        currencies
            .list(actor)
            .await
            .expect("list currencies")
            .iter()
            .any(|currency| currency.code == "SAR" && currency.is_enabled_for_organization)
    );
    currencies
        .set_enabled(actor, "usd", true)
        .await
        .expect("enable USD for organization");

    let settings =
        OrganizationSettingsService::new(SeaOrmOrganizationSettingsRepository::new(&database));
    let updated = settings
        .update(
            actor,
            OrganizationSettingsInput {
                name: " Updated organization ".to_string(),
                legal_name: Some(" Legal name ".to_string()),
                default_currency: "USD".to_string(),
                timezone: "America/New_York".to_string(),
            },
        )
        .await
        .expect("update organization settings");
    assert_eq!(updated.name, "Updated organization");
    assert_eq!(updated.legal_name.as_deref(), Some("Legal name"));
    assert_eq!(updated.default_currency, "USD");

    let profile = ProfileService::new(SeaOrmProfileRepository::new(&database))
        .update(
            actor,
            ProfileSettingsInput {
                name: "Updated Admin".to_string(),
                email: "UPDATED@example.test".to_string(),
                current_password: None,
                new_password: None,
            },
        )
        .await
        .expect("update profile");
    assert_eq!(profile.name, "Updated Admin");
    assert_eq!(profile.email, "updated@example.test");

    let templates = ContractTemplateService::new(SeaOrmContractTemplateRepository::new(&database));
    let created = templates
        .create(actor, template_input("Primary lease", None))
        .await
        .expect("create contract template");
    assert_eq!(created.versions.len(), 1);
    assert_eq!(created.versions[0].version_number, 1);
    assert_eq!(created.versions[0].content_sha256.len(), 64);

    let updated_template = templates
        .update(
            actor,
            &created.id,
            template_input("Primary lease v2", Some("Updated wording")),
        )
        .await
        .expect("create template version");
    assert_eq!(updated_template.versions.len(), 2);
    assert_eq!(updated_template.versions[0].version_number, 2);
    assert_eq!(
        updated_template.versions[0].change_note.as_deref(),
        Some("Updated wording")
    );

    templates
        .set_active(actor, &created.id, false)
        .await
        .expect("deactivate template");
    assert!(
        !templates
            .list(actor)
            .await
            .expect("list templates")
            .into_iter()
            .find(|template| template.id == created.id)
            .expect("created template")
            .is_active
    );

    let audit = AuditLog::new(SeaOrmAuditLogRepository::new(&database))
        .list(actor, 100, 0)
        .await
        .expect("read organization audit log");
    assert!(audit.len() >= 5);
    assert!(audit.iter().any(|entry| {
        entry.entity_type == "contract_template"
            && entry.actor_name.as_deref() == Some("Updated Admin")
    }));

    assert!(
        currencies.set_enabled(actor, "USD", false).await.is_err(),
        "the default currency cannot be disabled"
    );
}

fn template_input(name: &str, change_note: Option<&str>) -> ContractTemplateInput {
    ContractTemplateInput {
        template_key: "standard-residential".to_string(),
        name: name.to_string(),
        description: Some("Standard residential lease".to_string()),
        lease_type: "residential".to_string(),
        locale: "en".to_string(),
        content_format: "markdown".to_string(),
        content_body: format!("# {name}\nLease terms"),
        placeholders_json: Some(r#"{"tenant_name":"string"}"#.to_string()),
        change_note: change_note.map(str::to_string),
    }
}
