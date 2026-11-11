use super::execute_statements;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('AED', 'United Arab Emirates Dirham', 'درهم إماراتي', 'د.إ', 2) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('YER', 'Yemeni Rial', 'ريال يمني', 'ر.ي', 2) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('KWD', 'Kuwaiti Dinar', 'دينار كويتي', 'د.ك', 3) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('BHD', 'Bahraini Dinar', 'دينار بحريني', 'د.ب', 3) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('OMR', 'Omani Rial', 'ريال عماني', 'ر.ع', 3) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('QAR', 'Qatari Riyal', 'ريال قطري', 'ر.ق', 2) ON CONFLICT (code) DO NOTHING",
                "INSERT INTO currencies (code, name, name_ar, symbol, decimal_digits) VALUES ('EGP', 'Egyptian Pound', 'جنيه مصري', 'ج.م', 2) ON CONFLICT (code) DO NOTHING",
            ],
        )
        .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Onboarding currency defaults are retained to protect organization data.".to_string(),
        ))
    }
}
