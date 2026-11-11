use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(initial::Migration)]
    }
}

mod initial {
    use sea_orm_migration::prelude::*;

    #[derive(DeriveMigrationName)]
    pub struct Migration;

    #[async_trait::async_trait]
    impl MigrationTrait for Migration {
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("organizations"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(ColumnDef::new(Alias::new("name")).string().not_null())
                        .col(
                            ColumnDef::new(Alias::new("created_at"))
                                .string()
                                .not_null(),
                        )
                        .to_owned(),
                )
                .await?;

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("users"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("organization_id"))
                                .string()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Alias::new("name")).string().not_null())
                        .col(ColumnDef::new(Alias::new("email")).string().not_null())
                        .col(
                            ColumnDef::new(Alias::new("password_hash"))
                                .string()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Alias::new("role")).string().not_null())
                        .col(
                            ColumnDef::new(Alias::new("is_active"))
                                .boolean()
                                .not_null()
                                .default(true),
                        )
                        .col(
                            ColumnDef::new(Alias::new("created_at"))
                                .string()
                                .not_null(),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_users_organization")
                                .from(Alias::new("users"), Alias::new("organization_id"))
                                .to(Alias::new("organizations"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Cascade),
                        )
                        .to_owned(),
                )
                .await?;
            manager
                .create_index(
                    Index::create()
                        .name("uq_users_email")
                        .table(Alias::new("users"))
                        .col(Alias::new("email"))
                        .unique()
                        .to_owned(),
                )
                .await?;

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("properties"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("organization_id"))
                                .string()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Alias::new("name")).string().not_null())
                        .col(
                            ColumnDef::new(Alias::new("address"))
                                .string()
                                .not_null()
                                .default(""),
                        )
                        .col(
                            ColumnDef::new(Alias::new("property_type"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("created_at"))
                                .string()
                                .not_null(),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_properties_organization")
                                .from(Alias::new("properties"), Alias::new("organization_id"))
                                .to(Alias::new("organizations"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Cascade),
                        )
                        .to_owned(),
                )
                .await?;

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("units"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("property_id"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("unit_number"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("unit_type"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("bedrooms"))
                                .integer()
                                .not_null()
                                .default(0),
                        )
                        .col(
                            ColumnDef::new(Alias::new("bathrooms"))
                                .integer()
                                .not_null()
                                .default(0),
                        )
                        .col(
                            ColumnDef::new(Alias::new("rent_amount"))
                                .big_integer()
                                .not_null()
                                .default(0),
                        )
                        .col(
                            ColumnDef::new(Alias::new("availability_status"))
                                .string()
                                .not_null(),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_units_property")
                                .from(Alias::new("units"), Alias::new("property_id"))
                                .to(Alias::new("properties"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Cascade),
                        )
                        .to_owned(),
                )
                .await?;
            manager
                .create_index(
                    Index::create()
                        .name("uq_units_property_number")
                        .table(Alias::new("units"))
                        .col(Alias::new("property_id"))
                        .col(Alias::new("unit_number"))
                        .unique()
                        .to_owned(),
                )
                .await?;

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("tenants"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("organization_id"))
                                .string()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Alias::new("name")).string().not_null())
                        .col(ColumnDef::new(Alias::new("email")).string().null())
                        .col(ColumnDef::new(Alias::new("phone")).string().null())
                        .col(ColumnDef::new(Alias::new("national_id")).string().null())
                        .col(
                            ColumnDef::new(Alias::new("created_at"))
                                .string()
                                .not_null(),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_tenants_organization")
                                .from(Alias::new("tenants"), Alias::new("organization_id"))
                                .to(Alias::new("organizations"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Cascade),
                        )
                        .to_owned(),
                )
                .await?;

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("leases"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("unit_id"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("tenant_id"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("start_date"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("end_date"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("rent_amount"))
                                .big_integer()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("deposit_amount"))
                                .big_integer()
                                .not_null()
                                .default(0),
                        )
                        .col(ColumnDef::new(Alias::new("status")).string().not_null())
                        .col(
                            ColumnDef::new(Alias::new("created_at"))
                                .string()
                                .not_null(),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_leases_unit")
                                .from(Alias::new("leases"), Alias::new("unit_id"))
                                .to(Alias::new("units"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Restrict),
                        )
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_leases_tenant")
                                .from(Alias::new("leases"), Alias::new("tenant_id"))
                                .to(Alias::new("tenants"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Restrict),
                        )
                        .to_owned(),
                )
                .await?;

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new("payments"))
                        .if_not_exists()
                        .col(
                            ColumnDef::new(Alias::new("id"))
                                .string()
                                .not_null()
                                .primary_key(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("lease_id"))
                                .string()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("amount"))
                                .big_integer()
                                .not_null(),
                        )
                        .col(
                            ColumnDef::new(Alias::new("due_date"))
                                .string()
                                .not_null(),
                        )
                        .col(ColumnDef::new(Alias::new("paid_at")).string().null())
                        .col(ColumnDef::new(Alias::new("status")).string().not_null())
                        .col(
                            ColumnDef::new(Alias::new("payment_method"))
                                .string()
                                .null(),
                        )
                        .col(ColumnDef::new(Alias::new("reference")).string().null())
                        .foreign_key(
                            ForeignKey::create()
                                .name("fk_payments_lease")
                                .from(Alias::new("payments"), Alias::new("lease_id"))
                                .to(Alias::new("leases"), Alias::new("id"))
                                .on_delete(ForeignKeyAction::Cascade),
                        )
                        .to_owned(),
                )
                .await?;

            Ok(())
        }

        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
            for table in [
                "payments",
                "leases",
                "tenants",
                "units",
                "properties",
                "users",
                "organizations",
            ] {
                manager
                    .drop_table(
                        Table::drop()
                            .table(Alias::new(table))
                            .if_exists()
                            .to_owned(),
                    )
                    .await?;
            }
            Ok(())
        }
    }
}
