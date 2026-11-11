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
                r#"INSERT INTO currencies
                    (code, name, name_ar, symbol, symbol_position, decimal_digits)
                    VALUES
                    ('AED', 'UAE Dirham', 'درهم إماراتي', 'د.إ', 'suffix', 2),
                    ('AFN', 'Afghan Afghani', 'أفغاني', '؋', 'suffix', 2),
                    ('ALL', 'Albanian Lek', 'ليك ألباني', 'L', 'suffix', 2),
                    ('AMD', 'Armenian Dram', 'درام أرميني', '֏', 'suffix', 2),
                    ('ARS', 'Argentine Peso', 'بيزو أرجنتيني', '$', 'prefix', 2),
                    ('AUD', 'Australian Dollar', 'دولار أسترالي', 'A$', 'prefix', 2),
                    ('AZN', 'Azerbaijani Manat', 'مانات أذربيجاني', '₼', 'suffix', 2),
                    ('BHD', 'Bahraini Dinar', 'دينار بحريني', 'د.ب', 'suffix', 3),
                    ('BDT', 'Bangladeshi Taka', 'تاكا بنغلاديشي', '৳', 'suffix', 2),
                    ('BGN', 'Bulgarian Lev', 'ليف بلغاري', 'лв', 'suffix', 2),
                    ('BND', 'Brunei Dollar', 'دولار بروناي', 'B$', 'prefix', 2),
                    ('BOB', 'Bolivian Boliviano', 'بوليفيانو بوليفي', 'Bs.', 'suffix', 2),
                    ('BRL', 'Brazilian Real', 'ريال برازيلي', 'R$', 'prefix', 2),
                    ('CAD', 'Canadian Dollar', 'دولار كندي', 'C$', 'prefix', 2),
                    ('CHF', 'Swiss Franc', 'فرنك سويسري', 'CHF', 'prefix', 2),
                    ('CLP', 'Chilean Peso', 'بيزو تشيلي', '$', 'prefix', 0),
                    ('CNY', 'Chinese Yuan', 'يوان صيني', '¥', 'prefix', 2),
                    ('COP', 'Colombian Peso', 'بيزو كولومبي', '$', 'prefix', 2),
                    ('CZK', 'Czech Koruna', 'كرونة تشيكية', 'Kč', 'suffix', 2),
                    ('DKK', 'Danish Krone', 'كرونة دنماركية', 'kr', 'suffix', 2),
                    ('DZD', 'Algerian Dinar', 'دينار جزائري', 'د.ج', 'suffix', 2),
                    ('EGP', 'Egyptian Pound', 'جنيه مصري', 'ج.م', 'suffix', 2),
                    ('ETB', 'Ethiopian Birr', 'بير إثيوبي', 'Br', 'prefix', 2),
                    ('EUR', 'Euro', 'يورو', '€', 'prefix', 2),
                    ('GBP', 'Pound Sterling', 'جنيه إسترليني', '£', 'prefix', 2),
                    ('GEL', 'Georgian Lari', 'لاري جورجي', '₾', 'suffix', 2),
                    ('GHS', 'Ghanaian Cedi', 'سيدي غاني', '₵', 'prefix', 2),
                    ('HKD', 'Hong Kong Dollar', 'دولار هونغ كونغ', 'HK$', 'prefix', 2),
                    ('HUF', 'Hungarian Forint', 'فورنت مجري', 'Ft', 'suffix', 2),
                    ('IDR', 'Indonesian Rupiah', 'روبية إندونيسية', 'Rp', 'prefix', 2),
                    ('ILS', 'Israeli New Shekel', 'شيكل إسرائيلي', '₪', 'prefix', 2),
                    ('INR', 'Indian Rupee', 'روبية هندية', '₹', 'prefix', 2),
                    ('IQD', 'Iraqi Dinar', 'دينار عراقي', 'د.ع', 'suffix', 3),
                    ('IRR', 'Iranian Rial', 'ريال إيراني', '﷼', 'suffix', 2),
                    ('ISK', 'Icelandic Króna', 'كرونة آيسلندية', 'kr', 'suffix', 0),
                    ('JOD', 'Jordanian Dinar', 'دينار أردني', 'د.أ', 'suffix', 3),
                    ('JPY', 'Japanese Yen', 'ين ياباني', '¥', 'prefix', 0),
                    ('KES', 'Kenyan Shilling', 'شلن كيني', 'KSh', 'prefix', 2),
                    ('KRW', 'South Korean Won', 'وون كوري جنوبي', '₩', 'prefix', 0),
                    ('KWD', 'Kuwaiti Dinar', 'دينار كويتي', 'د.ك', 'suffix', 3),
                    ('KZT', 'Kazakhstani Tenge', 'تنغي كازاخستاني', '₸', 'suffix', 2),
                    ('LBP', 'Lebanese Pound', 'ليرة لبنانية', 'ل.ل', 'suffix', 2),
                    ('LKR', 'Sri Lankan Rupee', 'روبية سريلانكية', 'Rs', 'prefix', 2),
                    ('LYD', 'Libyan Dinar', 'دينار ليبي', 'د.ل', 'suffix', 3),
                    ('MAD', 'Moroccan Dirham', 'درهم مغربي', 'د.م', 'suffix', 2),
                    ('MMK', 'Myanmar Kyat', 'كيات ميانماري', 'K', 'suffix', 2),
                    ('MUR', 'Mauritian Rupee', 'روبية موريشية', '₨', 'prefix', 2),
                    ('MXN', 'Mexican Peso', 'بيزو مكسيكي', 'MX$', 'prefix', 2),
                    ('MYR', 'Malaysian Ringgit', 'رينغيت ماليزي', 'RM', 'prefix', 2),
                    ('NGN', 'Nigerian Naira', 'نايرا نيجيري', '₦', 'prefix', 2),
                    ('NOK', 'Norwegian Krone', 'كرونة نرويجية', 'kr', 'suffix', 2),
                    ('NPR', 'Nepalese Rupee', 'روبية نيبالية', '₨', 'prefix', 2),
                    ('NZD', 'New Zealand Dollar', 'دولار نيوزيلندي', 'NZ$', 'prefix', 2),
                    ('OMR', 'Omani Rial', 'ريال عماني', 'ر.ع', 'suffix', 3),
                    ('PEN', 'Peruvian Sol', 'سول بيروفي', 'S/', 'prefix', 2),
                    ('PHP', 'Philippine Peso', 'بيزو فلبيني', '₱', 'prefix', 2),
                    ('PKR', 'Pakistani Rupee', 'روبية باكستانية', '₨', 'prefix', 2),
                    ('PLN', 'Polish Złoty', 'زلوتي بولندي', 'zł', 'suffix', 2),
                    ('QAR', 'Qatari Riyal', 'ريال قطري', 'ر.ق', 'suffix', 2),
                    ('RON', 'Romanian Leu', 'ليو روماني', 'lei', 'suffix', 2),
                    ('RSD', 'Serbian Dinar', 'دينار صربي', 'дин.', 'suffix', 2),
                    ('RUB', 'Russian Ruble', 'روبل روسي', '₽', 'suffix', 2),
                    ('SAR', 'Saudi Riyal', 'ريال سعودي', 'ر.س', 'suffix', 2),
                    ('SEK', 'Swedish Krona', 'كرونة سويدية', 'kr', 'suffix', 2),
                    ('SGD', 'Singapore Dollar', 'دولار سنغافوري', 'S$', 'prefix', 2),
                    ('THB', 'Thai Baht', 'بات تايلندي', '฿', 'prefix', 2),
                    ('TND', 'Tunisian Dinar', 'دينار تونسي', 'د.ت', 'suffix', 3),
                    ('TRY', 'Turkish Lira', 'ليرة تركية', '₺', 'suffix', 2),
                    ('TWD', 'New Taiwan Dollar', 'دولار تايواني جديد', 'NT$', 'prefix', 2),
                    ('TZS', 'Tanzanian Shilling', 'شلن تنزاني', 'TSh', 'prefix', 2),
                    ('UAH', 'Ukrainian Hryvnia', 'هريفنيا أوكرانية', '₴', 'suffix', 2),
                    ('UGX', 'Ugandan Shilling', 'شلن أوغندي', 'USh', 'prefix', 0),
                    ('USD', 'US Dollar', 'دولار أمريكي', '$', 'prefix', 2),
                    ('UYU', 'Uruguayan Peso', 'بيزو أوروغواي', '$U', 'prefix', 2),
                    ('UZS', 'Uzbekistani Soʻm', 'سوم أوزبكستاني', 'soʻm', 'suffix', 2),
                    ('VND', 'Vietnamese Đồng', 'دونغ فيتنامي', '₫', 'suffix', 0),
                    ('XAF', 'Central African CFA Franc', 'فرنك وسط أفريقيا', 'FCFA', 'suffix', 0),
                    ('XOF', 'West African CFA Franc', 'فرنك غرب أفريقيا', 'CFA', 'suffix', 0),
                    ('ZAR', 'South African Rand', 'راند جنوب أفريقي', 'R', 'prefix', 2)
                    ON CONFLICT (code) DO NOTHING"#,
                r#"CREATE TABLE organization_currencies (
                    organization_id TEXT NOT NULL,
                    currency_code VARCHAR(3) NOT NULL,
                    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY (organization_id, currency_code),
                    CONSTRAINT fk_organization_currencies_organization
                        FOREIGN KEY (organization_id) REFERENCES organizations(id)
                        ON UPDATE CASCADE ON DELETE CASCADE,
                    CONSTRAINT fk_organization_currencies_currency
                        FOREIGN KEY (currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_organization_currencies_created_by
                        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                r#"CREATE TABLE exchange_rates (
                    id TEXT PRIMARY KEY NOT NULL,
                    organization_id TEXT NOT NULL,
                    base_currency_code VARCHAR(3) NOT NULL,
                    quote_currency_code VARCHAR(3) NOT NULL,
                    rate NUMERIC(24, 12) NOT NULL CHECK (rate > 0),
                    effective_at TIMESTAMPTZ NOT NULL,
                    source TEXT NOT NULL DEFAULT 'manual'
                        CHECK (source IN ('manual', 'import', 'provider')),
                    source_reference TEXT,
                    created_by TEXT,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (id, organization_id, base_currency_code, quote_currency_code),
                    UNIQUE (organization_id, base_currency_code, quote_currency_code, effective_at),
                    CONSTRAINT ck_exchange_rates_distinct_currencies
                        CHECK (base_currency_code <> quote_currency_code),
                    CONSTRAINT fk_exchange_rates_organization
                        FOREIGN KEY (organization_id) REFERENCES organizations(id)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_exchange_rates_base_currency
                        FOREIGN KEY (base_currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_exchange_rates_quote_currency
                        FOREIGN KEY (quote_currency_code) REFERENCES currencies(code)
                        ON UPDATE CASCADE ON DELETE RESTRICT,
                    CONSTRAINT fk_exchange_rates_created_by
                        FOREIGN KEY (created_by, organization_id) REFERENCES users(id, organization_id)
                        ON UPDATE CASCADE ON DELETE RESTRICT
                )"#,
                "CREATE INDEX idx_exchange_rates_lookup ON exchange_rates (organization_id, base_currency_code, quote_currency_code, effective_at)",
                "ALTER TABLE payment_allocations ADD COLUMN exchange_rate_id TEXT REFERENCES exchange_rates(id) ON UPDATE CASCADE ON DELETE RESTRICT",
                "CREATE INDEX idx_payment_allocations_exchange_rate ON payment_allocations (exchange_rate_id) WHERE exchange_rate_id IS NOT NULL",
            ],
        )
        .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_statements(
            manager,
            &[
                "DROP INDEX idx_payment_allocations_exchange_rate",
                "ALTER TABLE payment_allocations DROP COLUMN exchange_rate_id",
                "DROP TABLE exchange_rates",
                "DROP TABLE organization_currencies",
            ],
        )
        .await
    }
}
