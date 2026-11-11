use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use keyring::{Entry, Error as KeyringError};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc, time::Duration as StdDuration};
use tokio::sync::{Mutex, RwLock};
use url::Url;
use uuid::Uuid;

const TRIAL_IDENTITY_SERVICE: &str = "com.aqardesk.desktop.trial";
const DEFAULT_API_URL: &str = "https://api.mulverse.com";
const DEFAULT_VERIFY_INTERVAL_HOURS: i64 = 24;
const EXPIRING_WITHIN_DAYS: i64 = 14;

pub const ENTITLEMENTS: &[&str] = &[
    "properties",
    "units",
    "tenants",
    "payments",
    "contracts",
    "user_management",
    "contract_templates",
    "audit_log",
    "multi_currency",
];

pub const LIMIT_KEYS: &[(&str, &str)] = &[
    ("properties.max", "properties"),
    ("units.max", "units"),
    ("tenants.max", "tenants"),
    ("users.max", "users"),
    ("currencies.max", "currencies"),
    ("contract_templates.max", "templates"),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LicenseState {
    Unavailable,
    Pending,
    Trial,
    Active,
    Expiring,
    OfflineGrace,
    DeactivationPending,
    Expired,
    Suspended,
    Revoked,
    Invalid,
}

impl LicenseState {
    fn is_operational(&self) -> bool {
        matches!(
            self,
            Self::Trial | Self::Active | Self::Expiring | Self::OfflineGrace
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitEntry {
    pub key: String,
    pub max_value: Option<i64>,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStateDto {
    pub state: LicenseState,
    pub product: String,
    pub plan: Option<String>,
    pub plan_name: Option<String>,
    pub organization_id: Option<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub days_until_expiry: Option<i64>,
    pub trial_days_remaining: i64,
    pub is_operational: bool,
    pub is_restricted: bool,
    pub is_configured: bool,
    pub configuration_error: Option<String>,
    pub last_error: Option<String>,
    pub installation_id: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub enabled_features: Vec<String>,
    pub trial_features: Vec<String>,
    pub limits: Vec<LimitEntry>,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEntitlement {
    pub v: u32,
    pub kid: String,
    pub alg: String,
    pub payload: EntitlementPayload,
    pub sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementPayload {
    pub license_id: String,
    #[serde(rename = "application_key", alias = "product_key")]
    pub product_key: String,
    #[serde(rename = "application_id", alias = "product_id")]
    pub product_id: String,
    #[serde(rename = "customer_id", alias = "organization_id")]
    pub organization_id: String,
    pub edition_key: Option<String>,
    pub plan_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_name: Option<String>,
    pub entitlements: Vec<EntitlementItem>,
    pub limits: Vec<LimitItem>,
    pub activation_id: String,
    pub installation_id: String,
    pub issued_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub offline_allowed: bool,
    pub offline_grace_period_days: Option<i32>,
    pub verification_interval_hours: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementItem {
    pub key: String,
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitItem {
    pub key: String,
    pub max_value: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseSnapshot {
    state: LicenseState,
    signed_entitlement: Option<SignedEntitlement>,
    installation_id: String,
    app_version: String,
    last_sync_at: Option<DateTime<Utc>>,
    last_known_time: DateTime<Utc>,
    server_status: Option<String>,
    last_error: Option<String>,
}

impl LicenseSnapshot {
    fn empty(installation_id: String, app_version: String) -> Self {
        Self {
            state: LicenseState::Unavailable,
            signed_entitlement: None,
            installation_id,
            app_version,
            last_sync_at: None,
            last_known_time: Utc::now(),
            server_status: None,
            last_error: None,
        }
    }
}

#[derive(Clone)]
struct LicenseConfig {
    api_base_url: String,
    product_code: String,
    product_id: String,
    key_prefix: String,
    public_key_id: String,
    public_key_b64: String,
    error: Option<String>,
}

impl LicenseConfig {
    fn from_build_env() -> Self {
        let api_base_url = option_env!("MULVERSE_LICENSE_API_BASE_URL")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(DEFAULT_API_URL)
            .trim_end_matches('/')
            .to_string();
        let product_code = option_env!("MULVERSE_LICENSE_PRODUCT_CODE")
            .unwrap_or_default()
            .trim()
            .to_string();
        let product_id = option_env!("MULVERSE_LICENSE_PRODUCT_ID")
            .unwrap_or_default()
            .trim()
            .to_string();
        let product_id = Uuid::parse_str(&product_id)
            .map(|value| value.to_string())
            .unwrap_or(product_id);
        let key_prefix = option_env!("MULVERSE_LICENSE_KEY_PREFIX")
            .unwrap_or_default()
            .trim()
            .to_ascii_uppercase();
        let key_prefix = if key_prefix.is_empty() {
            key_prefix
        } else {
            format!("{}-", key_prefix.trim_end_matches('-'))
        };
        let public_key_id = option_env!("MULVERSE_LICENSE_PUBLIC_KEY_ID")
            .unwrap_or_default()
            .trim()
            .to_string();
        let public_key_b64 = option_env!("MULVERSE_LICENSE_PUBLIC_KEY_B64")
            .unwrap_or_default()
            .trim()
            .to_string();

        let error = if product_code.is_empty()
            || Uuid::parse_str(&product_id).is_err()
            || key_prefix.is_empty()
            || public_key_id.is_empty()
            || public_key_b64.is_empty()
        {
            Some("Mulverse product identity and public verification key are not configured.".into())
        } else if Url::parse(&api_base_url).map_or(true, |url| url.scheme() != "https") {
            Some("The Mulverse licensing API must use HTTPS.".into())
        } else if STANDARD
            .decode(&public_key_b64)
            .map_or(true, |bytes| bytes.len() != 32)
        {
            Some("The configured Ed25519 public key is invalid.".into())
        } else {
            None
        };

        Self {
            api_base_url,
            product_code,
            product_id,
            key_prefix,
            public_key_id,
            public_key_b64,
            error,
        }
    }

    fn is_configured(&self) -> bool {
        self.error.is_none()
    }
}

#[derive(Debug, Serialize)]
struct ActivationRequest {
    #[serde(rename = "application_key")]
    product_key: String,
    #[serde(rename = "customer_id")]
    organization_id: String,
    installation_id: String,
    platform: String,
    app_version: String,
}

#[derive(Debug, Deserialize)]
struct ActivationResponse {
    activation: ActivationInfo,
    signed_entitlement: SignedEntitlement,
}

#[derive(Debug, Serialize)]
struct ClaimProductKeyRequest {
    #[serde(rename = "application_key")]
    product_key: String,
    #[serde(rename = "customer_id")]
    organization_id: String,
    customer_name: String,
    customer_slug: String,
    customer_profile: TrialCustomerProfile,
}

#[derive(Debug, Deserialize)]
struct ClaimProductKeyResponse {
    #[serde(alias = "organization_id")]
    customer_id: String,
}

#[derive(Debug, Deserialize)]
struct ActivationInfo {
    license_id: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct RefreshRequest {
    license_id: String,
    installation_id: String,
    app_version: String,
}

#[derive(Debug, Serialize)]
struct DeactivationRequest {
    license_id: String,
    installation_id: String,
}

#[derive(Debug, Deserialize)]
struct TrialChallenge {
    challenge_id: String,
    challenge: String,
}

#[derive(Debug, Serialize)]
struct TrialCustomerProfile {
    name_ar: Option<String>,
    institution_type: String,
    customer_size: Option<String>,
    educational_track: Option<String>,
    country_code: Option<String>,
    country_name: Option<String>,
    region_name: Option<String>,
    city: Option<String>,
}

#[derive(Debug, Serialize)]
struct RegisterTrialRequest {
    device_public_key: String,
    device_signature: String,
    challenge_id: String,
    challenge: String,
    installation_id: String,
    customer_id: String,
    customer_name: String,
    customer_slug: String,
    customer_profile: TrialCustomerProfile,
    platform: String,
    app_version: String,
    machine_label: Option<String>,
    application_id: String,
}

#[derive(Debug, Serialize)]
struct RefreshTrialRequest {
    trial_id: String,
    device_public_key: String,
    device_signature: String,
    challenge_id: String,
    challenge: String,
    installation_id: String,
    customer_id: String,
    app_version: String,
    application_id: String,
}

#[derive(Debug, Deserialize)]
struct TrialRegistrationResponse {
    trial: TrialDevice,
    signed_entitlement: SignedEntitlement,
}

#[derive(Debug, Deserialize)]
struct TrialDevice {
    id: String,
    status: String,
    installation_id: String,
}

#[derive(Debug, Serialize)]
struct LegacyEntitlementPayload<'a> {
    license_id: &'a str,
    product_key: &'a str,
    product_id: &'a str,
    organization_id: &'a str,
    edition_key: &'a Option<String>,
    plan_key: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_name: &'a Option<String>,
    entitlements: &'a [EntitlementItem],
    limits: &'a [LimitItem],
    activation_id: &'a str,
    installation_id: &'a str,
    issued_at: &'a DateTime<Utc>,
    valid_from: &'a DateTime<Utc>,
    valid_until: &'a Option<DateTime<Utc>>,
    offline_allowed: bool,
    offline_grace_period_days: Option<i32>,
    verification_interval_hours: Option<i32>,
}

pub struct LicenseRuntime {
    config: LicenseConfig,
    app_data_dir: PathBuf,
    client: reqwest::Client,
    snapshot: RwLock<LicenseSnapshot>,
    lifecycle_lock: Mutex<()>,
}

impl LicenseRuntime {
    pub fn new(app_data_dir: PathBuf, app_version: String) -> Result<Self, String> {
        let config = LicenseConfig::from_build_env();
        let installation_id = load_or_create_installation_id(&app_data_dir)?;
        let client = reqwest::Client::builder()
            .user_agent("AqarDesk-Desktop")
            .timeout(StdDuration::from_secs(15))
            .connect_timeout(StdDuration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| {
                tracing::error!(%error, "Could not initialize the license network client");
                "The license network client could not be initialized.".to_string()
            })?;
        let state_path = app_data_dir.join("license-state.json");
        let snapshot = match std::fs::read(&state_path) {
            Ok(bytes) => match serde_json::from_slice::<LicenseSnapshot>(&bytes) {
                Ok(mut snapshot) => {
                    if snapshot.installation_id != installation_id {
                        snapshot.state = LicenseState::Invalid;
                        snapshot.last_error =
                            Some("The saved license is bound to another installation.".into());
                    }
                    if let Err(error) = Self::verify_signed_entitlement(&config, &snapshot) {
                        snapshot.state = LicenseState::Invalid;
                        snapshot.last_error = Some(error);
                    }
                    snapshot
                }
                Err(error) => {
                    tracing::error!(%error, "Could not read the persisted license state");
                    let mut snapshot =
                        LicenseSnapshot::empty(installation_id.clone(), app_version.clone());
                    snapshot.state = LicenseState::Invalid;
                    snapshot.last_error = Some(
                        "The saved license data is corrupt and must be activated again.".into(),
                    );
                    snapshot
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                LicenseSnapshot::empty(installation_id.clone(), app_version.clone())
            }
            Err(error) => {
                tracing::error!(%error, "Could not read the persisted license state");
                let mut snapshot =
                    LicenseSnapshot::empty(installation_id.clone(), app_version.clone());
                snapshot.state = LicenseState::Invalid;
                snapshot.last_error = Some("The saved license data could not be read.".into());
                snapshot
            }
        };
        Ok(Self {
            config,
            app_data_dir,
            client,
            snapshot: RwLock::new(snapshot),
            lifecycle_lock: Mutex::new(()),
        })
    }

    pub async fn state(&self, organization_id: Option<&str>) -> LicenseStateDto {
        let mut snapshot = self.snapshot.read().await.clone();
        self.evaluate(&mut snapshot);
        let org_matches = organization_id.map_or(false, |organization_id| {
            snapshot
                .signed_entitlement
                .as_ref()
                .is_some_and(|entitlement| {
                    same_uuid(&entitlement.payload.organization_id, organization_id)
                })
        });
        let state = if self.config.is_configured() {
            snapshot.state.clone()
        } else {
            LicenseState::Unavailable
        };
        let entitlement = snapshot.signed_entitlement.as_ref();
        let enabled_features = entitlement
            .map(|entitlement| {
                entitlement
                    .payload
                    .entitlements
                    .iter()
                    .filter(|entry| entry.granted)
                    .map(|entry| entry.key.clone())
                    .collect()
            })
            .unwrap_or_default();
        let limits = entitlement
            .map(|entitlement| {
                entitlement
                    .payload
                    .limits
                    .iter()
                    .map(|limit| LimitEntry {
                        key: limit.key.clone(),
                        max_value: limit.max_value.filter(|value| *value >= 0),
                        unit: limit_unit(&limit.key).to_string(),
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                LIMIT_KEYS
                    .iter()
                    .map(|(key, unit)| LimitEntry {
                        key: (*key).to_string(),
                        max_value: None,
                        unit: (*unit).to_string(),
                    })
                    .collect()
            });
        let days_until_expiry = entitlement
            .and_then(|entitlement| entitlement.payload.valid_until)
            .map(|valid_until| {
                valid_until
                    .signed_duration_since(Utc::now())
                    .num_days()
                    .max(0)
            });
        let trial_days_remaining = entitlement
            .filter(|entitlement| is_trial(entitlement.payload.plan_key.as_deref()))
            .and_then(|entitlement| entitlement.payload.valid_until)
            .map(|valid_until| {
                let seconds = valid_until.signed_duration_since(Utc::now()).num_seconds();
                if seconds <= 0 {
                    0
                } else {
                    (seconds + 86_399) / 86_400
                }
            })
            .unwrap_or(0);
        let is_operational = state.is_operational() && org_matches;

        LicenseStateDto {
            state: state.clone(),
            product: if self.config.product_code.is_empty() {
                "AqarDesk".into()
            } else {
                self.config.product_code.clone()
            },
            plan: entitlement.and_then(|entitlement| entitlement.payload.plan_key.clone()),
            plan_name: entitlement.and_then(|entitlement| entitlement.payload.plan_name.clone()),
            organization_id: entitlement
                .map(|entitlement| entitlement.payload.organization_id.clone()),
            valid_until: entitlement.and_then(|entitlement| entitlement.payload.valid_until),
            days_until_expiry,
            trial_days_remaining,
            is_operational,
            is_restricted: !is_operational,
            is_configured: self.config.is_configured(),
            configuration_error: self.config.error.clone(),
            last_error: snapshot.last_error,
            installation_id: snapshot.installation_id,
            last_sync_at: snapshot.last_sync_at,
            enabled_features,
            trial_features: ENTITLEMENTS.iter().map(|key| (*key).to_string()).collect(),
            limits,
            app_version: snapshot.app_version,
        }
    }

    pub async fn require_operational(&self, organization_id: &str) -> Result<(), String> {
        let snapshot = self.snapshot.read().await.clone();
        self.authorize_snapshot(&snapshot, organization_id, None)
    }

    pub async fn require_feature(
        &self,
        organization_id: &str,
        feature: &str,
    ) -> Result<(), String> {
        let snapshot = self.snapshot.read().await.clone();
        self.authorize_snapshot(&snapshot, organization_id, Some(feature))
    }

    pub async fn check_limit_for_organization(
        &self,
        organization_id: &str,
        key: &str,
        current_count: u64,
        adding: u64,
    ) -> Result<(), String> {
        let snapshot = self.snapshot.read().await.clone();
        self.authorize_snapshot(&snapshot, organization_id, None)?;
        let entitlement = snapshot
            .signed_entitlement
            .as_ref()
            .expect("operational licenses must have an entitlement");
        let maximum = entitlement
            .payload
            .limits
            .iter()
            .find(|limit| limit.key == key)
            .and_then(|limit| limit.max_value)
            .filter(|maximum| *maximum >= 0);
        if let Some(maximum) = maximum {
            let new_total = current_count.saturating_add(adding);
            if new_total > maximum as u64 {
                return Err(format!(
                    "License limit exceeded for {key} ({current_count} + {adding}, maximum {maximum})."
                ));
            }
        }
        Ok(())
    }

    pub async fn activate(
        &self,
        product_key: &str,
        organization_id: &str,
        organization_name: &str,
    ) -> Result<LicenseStateDto, String> {
        let _lifecycle_guard = self.lifecycle_lock.lock().await;
        self.ensure_configured()?;
        let organization_id = Uuid::parse_str(organization_id)
            .map_err(|_| "The active organization identifier is invalid.".to_string())?;
        let product_key = product_key.trim().to_ascii_uppercase();
        self.validate_product_key(&product_key)?;
        let current = self.snapshot.read().await.clone();
        let request = ActivationRequest {
            product_key: product_key.clone(),
            organization_id: organization_id.to_string(),
            installation_id: current.installation_id.clone(),
            platform: std::env::consts::OS.to_string(),
            app_version: current.app_version.clone(),
        };
        let response: ActivationResponse = match self
            .post("/api/v1/public/licenses/activate", &request)
            .await
        {
            Ok(response) => response,
            Err(error) if application_key_requires_claim(&error) => {
                let customer_slug = customer_slug(organization_id, organization_name);
                let claim = ClaimProductKeyRequest {
                    product_key,
                    organization_id: organization_id.to_string(),
                    customer_name: organization_name.to_string(),
                    customer_slug,
                    customer_profile: TrialCustomerProfile {
                        name_ar: None,
                        institution_type: "real_estate".into(),
                        customer_size: None,
                        educational_track: None,
                        country_code: None,
                        country_name: None,
                        region_name: None,
                        city: None,
                    },
                };
                let claim_response: ClaimProductKeyResponse =
                    self.post("/api/v1/public/keys/claim", &claim).await?;
                if !same_uuid(&claim_response.customer_id, &organization_id.to_string()) {
                    return Err(
                        "Mulverse claimed the product key for a different organization.".into(),
                    );
                }
                self.post("/api/v1/public/licenses/activate", &request)
                    .await?
            }
            Err(error) => return Err(error),
        };
        self.validate_entitlement(&response.signed_entitlement, &current.installation_id)?;
        if response.activation.license_id != response.signed_entitlement.payload.license_id
            || !same_uuid(
                &response.signed_entitlement.payload.organization_id,
                &organization_id.to_string(),
            )
        {
            return Err("The activated license does not match this organization.".into());
        }
        let mut snapshot = current;
        snapshot.signed_entitlement = Some(response.signed_entitlement);
        snapshot.server_status = Some(response.activation.status);
        snapshot.last_sync_at = Some(Utc::now());
        snapshot.last_error = None;
        snapshot.state = LicenseState::Unavailable;
        self.evaluate(&mut snapshot);
        self.store(snapshot.clone()).await?;
        Ok(self.state(Some(&organization_id.to_string())).await)
    }

    pub async fn start_trial(
        &self,
        organization_id: &str,
        organization_name: &str,
    ) -> Result<LicenseStateDto, String> {
        let _lifecycle_guard = self.lifecycle_lock.lock().await;
        self.ensure_configured()?;
        let organization_id = Uuid::parse_str(organization_id)
            .map_err(|_| "The active organization identifier is invalid.".to_string())?;
        let current = self.snapshot.read().await.clone();
        if current.signed_entitlement.is_some() {
            return Err("Deactivate the current license before starting a trial.".into());
        }
        let challenge: TrialChallenge = self
            .post("/api/v1/public/trials/challenge", &serde_json::json!({}))
            .await?;
        let slug = customer_slug(organization_id, organization_name);
        let profile = TrialCustomerProfile {
            name_ar: None,
            institution_type: "real_estate".into(),
            customer_size: None,
            educational_track: None,
            country_code: None,
            country_name: None,
            region_name: None,
            city: None,
        };
        let fields = vec![
            "register".to_string(),
            challenge.challenge_id.clone(),
            current.installation_id.clone(),
            organization_id.to_string(),
            organization_name.to_string(),
            slug.clone(),
            String::new(),
            "real_estate".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            std::env::consts::OS.to_string(),
            current.app_version.clone(),
            self.config.product_id.clone(),
        ];
        let (device_public_key, device_signature) =
            self.sign_trial_challenge(&challenge, &fields, &current.installation_id)?;
        let request = RegisterTrialRequest {
            device_public_key,
            device_signature,
            challenge_id: challenge.challenge_id,
            challenge: challenge.challenge,
            installation_id: current.installation_id.clone(),
            customer_id: organization_id.to_string(),
            customer_name: organization_name.to_string(),
            customer_slug: slug,
            customer_profile: profile,
            platform: std::env::consts::OS.to_string(),
            app_version: current.app_version.clone(),
            machine_label: None,
            application_id: self.config.product_id.clone(),
        };
        let response: TrialRegistrationResponse = self
            .post("/api/v1/public/trials/register", &request)
            .await?;
        self.store_trial_response(&current, response, &organization_id.to_string())
            .await?;
        Ok(self.state(Some(&organization_id.to_string())).await)
    }

    pub async fn refresh(&self, organization_id: Option<&str>) -> Result<LicenseStateDto, String> {
        let _lifecycle_guard = self.lifecycle_lock.lock().await;
        self.ensure_configured()?;
        let current = self.snapshot.read().await.clone();
        if current.state == LicenseState::DeactivationPending {
            return Err(
                "License deactivation is pending; retry deactivation before refreshing.".into(),
            );
        }
        let Some(entitlement) = current.signed_entitlement.clone() else {
            return Ok(self.state(organization_id).await);
        };
        if let Some(expected) = organization_id {
            if !same_uuid(&entitlement.payload.organization_id, expected) {
                return Err("The license is not valid for the active organization.".into());
            }
        }
        if is_trial(entitlement.payload.plan_key.as_deref()) {
            let organization_id = organization_id.unwrap_or(&entitlement.payload.organization_id);
            return self
                .refresh_trial(&current, &entitlement, organization_id)
                .await;
        }
        let request = RefreshRequest {
            license_id: entitlement.payload.license_id.clone(),
            installation_id: current.installation_id.clone(),
            app_version: current.app_version.clone(),
        };
        let response: ActivationResponse = self
            .post("/api/v1/public/licenses/refresh", &request)
            .await?;
        self.validate_entitlement(&response.signed_entitlement, &current.installation_id)?;
        if response.activation.license_id != response.signed_entitlement.payload.license_id
            || !same_uuid(
                &response.signed_entitlement.payload.organization_id,
                &entitlement.payload.organization_id,
            )
        {
            return Err("Mulverse returned a license for a different organization.".into());
        }
        let mut snapshot = current;
        snapshot.signed_entitlement = Some(response.signed_entitlement);
        snapshot.server_status = Some(response.activation.status);
        snapshot.last_sync_at = Some(Utc::now());
        snapshot.last_error = None;
        snapshot.state = LicenseState::Unavailable;
        self.evaluate(&mut snapshot);
        self.store(snapshot).await?;
        Ok(self.state(organization_id).await)
    }

    pub async fn deactivate_for_organization(
        &self,
        organization_id: &str,
    ) -> Result<LicenseStateDto, String> {
        let _lifecycle_guard = self.lifecycle_lock.lock().await;
        self.ensure_configured()?;
        let current = self.snapshot.read().await.clone();
        if let Some(entitlement) = &current.signed_entitlement {
            if !same_uuid(&entitlement.payload.organization_id, organization_id) {
                return Err("The AqarDesk license does not match the active organization.".into());
            }
            if !is_trial(entitlement.payload.plan_key.as_deref()) {
                let request = DeactivationRequest {
                    license_id: entitlement.payload.license_id.clone(),
                    installation_id: current.installation_id.clone(),
                };
                if let Err(error) = self
                    .post_empty("/api/v1/public/licenses/deactivate", &request)
                    .await
                {
                    let mut pending = current;
                    pending.state = LicenseState::DeactivationPending;
                    pending.last_error = Some(error.clone());
                    self.store(pending).await?;
                    return Err(error);
                }
            }
        }
        let snapshot = LicenseSnapshot::empty(current.installation_id, current.app_version);
        self.store(snapshot).await?;
        Ok(self.state(Some(organization_id)).await)
    }

    async fn refresh_trial(
        &self,
        current: &LicenseSnapshot,
        entitlement: &SignedEntitlement,
        organization_id: &str,
    ) -> Result<LicenseStateDto, String> {
        let challenge: TrialChallenge = self
            .post("/api/v1/public/trials/challenge", &serde_json::json!({}))
            .await?;
        let fields = vec![
            "refresh".to_string(),
            challenge.challenge_id.clone(),
            entitlement.payload.license_id.clone(),
            current.installation_id.clone(),
            organization_id.to_string(),
            current.app_version.clone(),
            self.config.product_id.clone(),
        ];
        let (device_public_key, device_signature) =
            self.sign_trial_challenge(&challenge, &fields, &current.installation_id)?;
        let request = RefreshTrialRequest {
            trial_id: entitlement.payload.license_id.clone(),
            device_public_key,
            device_signature,
            challenge_id: challenge.challenge_id,
            challenge: challenge.challenge,
            installation_id: current.installation_id.clone(),
            customer_id: organization_id.to_string(),
            app_version: current.app_version.clone(),
            application_id: self.config.product_id.clone(),
        };
        let response: TrialRegistrationResponse =
            self.post("/api/v1/public/trials/refresh", &request).await?;
        self.store_trial_response(current, response, organization_id)
            .await?;
        Ok(self.state(Some(organization_id)).await)
    }

    async fn store_trial_response(
        &self,
        current: &LicenseSnapshot,
        response: TrialRegistrationResponse,
        organization_id: &str,
    ) -> Result<(), String> {
        if response.trial.status != "active"
            || response.trial.installation_id != current.installation_id
            || response.trial.id != response.signed_entitlement.payload.license_id
            || !is_trial(response.signed_entitlement.payload.plan_key.as_deref())
            || !same_uuid(
                &response.signed_entitlement.payload.organization_id,
                organization_id,
            )
        {
            return Err("Mulverse returned an inactive or mismatched trial.".into());
        }
        self.validate_entitlement(&response.signed_entitlement, &current.installation_id)?;
        let mut snapshot = current.clone();
        snapshot.signed_entitlement = Some(response.signed_entitlement);
        snapshot.server_status = Some(response.trial.status);
        snapshot.last_sync_at = Some(Utc::now());
        snapshot.last_error = None;
        snapshot.state = LicenseState::Unavailable;
        self.evaluate(&mut snapshot);
        self.store(snapshot).await
    }

    fn sign_trial_challenge(
        &self,
        challenge: &TrialChallenge,
        fields: &[String],
        installation_id: &str,
    ) -> Result<(String, String), String> {
        let entry = Entry::new(TRIAL_IDENTITY_SERVICE, installation_id)
            .map_err(|error| format!("Could not access the OS key store: {error}"))?;
        let key_bytes = match entry.get_password() {
            Ok(value) => STANDARD
                .decode(value)
                .map_err(|_| "The saved trial device key is invalid.".to_string())?,
            Err(KeyringError::NoEntry) => {
                let mut seed = [0_u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut seed);
                entry
                    .set_password(&STANDARD.encode(seed))
                    .map_err(|error| {
                        format!("Could not securely save the trial device key: {error}")
                    })?;
                seed.to_vec()
            }
            Err(error) => return Err(format!("Could not access the OS key store: {error}")),
        };
        let seed: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| "The saved trial device key has an invalid length.".to_string())?;
        let signing_key = SigningKey::from_bytes(&seed);
        let mut message = String::from("mulverse-trial-v1");
        for field in fields {
            message.push('\n');
            message.push_str(field);
        }
        message.push('\n');
        message.push_str(&challenge.challenge);
        let signature = signing_key.sign(message.as_bytes());
        Ok((
            STANDARD.encode(signing_key.verifying_key().to_bytes()),
            STANDARD.encode(signature.to_bytes()),
        ))
    }

    async fn post<TRequest: Serialize + ?Sized, TResponse: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        request: &TRequest,
    ) -> Result<TResponse, String> {
        let url = format!("{}{}", self.config.api_base_url, path);
        let response = self
            .client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "Mulverse license request failed");
                format!("Could not reach Mulverse licensing service: {error}")
            })?;
        let status = response.status();
        let body = response.text().await.map_err(|error| {
            tracing::error!(%error, "Could not read Mulverse license response");
            "Could not read the Mulverse licensing response.".to_string()
        })?;
        if !status.is_success() {
            let message = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .get("error")
                        .and_then(|error| error.get("message").or(Some(error)))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| format!("Mulverse request failed with HTTP {status}."));
            return Err(message);
        }
        serde_json::from_str(&body).map_err(|error| {
            tracing::error!(%error, "Could not parse Mulverse license response");
            "Mulverse returned an invalid licensing response.".to_string()
        })
    }

    async fn post_empty<TRequest: Serialize + ?Sized>(
        &self,
        path: &str,
        request: &TRequest,
    ) -> Result<(), String> {
        let url = format!("{}{}", self.config.api_base_url, path);
        let response = self
            .client
            .post(url)
            .json(request)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(%error, "Mulverse license request failed");
                format!("Could not reach Mulverse licensing service: {error}")
            })?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.map_err(|error| {
                tracing::error!(%error, status = %status, "Could not read Mulverse deactivation error");
                format!("Mulverse request failed with HTTP {status}; its error response could not be read.")
            })?;
            let message = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .get("error")
                        .and_then(|error| error.get("message").or(Some(error)))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
                .filter(|message| !message.is_empty())
                .unwrap_or_else(|| format!("Mulverse request failed with HTTP {status}."));
            Err(message)
        }
    }

    fn validate_product_key(&self, product_key: &str) -> Result<(), String> {
        let segments = product_key
            .strip_prefix(&self.config.key_prefix)
            .map(|suffix| suffix.split('-').collect::<Vec<_>>());
        if !segments.as_ref().is_some_and(|segments| {
            segments.len() == 4
                && segments.iter().all(|segment| {
                    segment.len() == 5 && segment.bytes().all(|byte| byte.is_ascii_alphanumeric())
                })
        }) {
            return Err(format!(
                "Enter a valid product key beginning with {}.",
                self.config.key_prefix
            ));
        }
        Ok(())
    }

    fn validate_entitlement(
        &self,
        entitlement: &SignedEntitlement,
        installation_id: &str,
    ) -> Result<(), String> {
        Self::verify_signed_entitlement(
            &self.config,
            &LicenseSnapshot {
                signed_entitlement: Some(entitlement.clone()),
                installation_id: installation_id.to_string(),
                ..LicenseSnapshot::empty(installation_id.to_string(), String::new())
            },
        )
    }

    fn verify_signed_entitlement(
        config: &LicenseConfig,
        snapshot: &LicenseSnapshot,
    ) -> Result<(), String> {
        let Some(entitlement) = &snapshot.signed_entitlement else {
            return Ok(());
        };
        if !config.is_configured() {
            return Err(config
                .error
                .clone()
                .unwrap_or_else(|| "Mulverse licensing is not configured.".to_string()));
        }
        if entitlement.alg != "Ed25519" {
            return Err(
                "Mulverse returned a license with an unsupported signature algorithm.".into(),
            );
        }
        if entitlement.kid != config.public_key_id {
            return Err(
                "Mulverse signed the license with a different key ID. Check MULVERSE_LICENSE_PUBLIC_KEY_ID."
                    .into(),
            );
        }
        if !same_uuid(&entitlement.payload.product_id, &config.product_id) {
            return Err(
                "Mulverse returned a license for a different product. Check MULVERSE_LICENSE_PRODUCT_ID."
                    .into(),
            );
        }
        if entitlement.payload.installation_id != snapshot.installation_id {
            return Err(
                "Mulverse returned a license for a different installation. Retry from the same AqarDesk installation."
                    .into(),
            );
        }
        let public_key_bytes = STANDARD
            .decode(&config.public_key_b64)
            .map_err(|_| "The configured license verification key is invalid.".to_string())?;
        let key_bytes: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| "The configured license verification key is invalid.".to_string())?;
        let public_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|_| "The configured license verification key is invalid.".to_string())?;
        let signature_bytes = STANDARD
            .decode(&entitlement.sig)
            .map_err(|_| "The license signature is invalid.".to_string())?;
        let signature = Signature::from_slice(&signature_bytes)
            .map_err(|_| "The license signature is invalid.".to_string())?;
        let canonical = serde_json::to_vec(&entitlement.payload)
            .map_err(|_| "The license data could not be verified.".to_string())?;
        if public_key.verify(&canonical, &signature).is_err() {
            let payload = &entitlement.payload;
            let legacy = LegacyEntitlementPayload {
                license_id: &payload.license_id,
                product_key: &payload.product_key,
                product_id: &payload.product_id,
                organization_id: &payload.organization_id,
                edition_key: &payload.edition_key,
                plan_key: &payload.plan_key,
                plan_name: &payload.plan_name,
                entitlements: &payload.entitlements,
                limits: &payload.limits,
                activation_id: &payload.activation_id,
                installation_id: &payload.installation_id,
                issued_at: &payload.issued_at,
                valid_from: &payload.valid_from,
                valid_until: &payload.valid_until,
                offline_allowed: payload.offline_allowed,
                offline_grace_period_days: payload.offline_grace_period_days,
                verification_interval_hours: payload.verification_interval_hours,
            };
            let legacy = serde_json::to_vec(&legacy)
                .map_err(|_| "The license data could not be verified.".to_string())?;
            public_key
                .verify(&legacy, &signature)
                .map_err(|_| "The license signature could not be verified.".to_string())?;
        }
        Ok(())
    }

    fn evaluate(&self, snapshot: &mut LicenseSnapshot) {
        if snapshot.state == LicenseState::DeactivationPending {
            return;
        }
        if !self.config.is_configured() {
            snapshot.state = LicenseState::Unavailable;
            return;
        }
        let Some(entitlement) = snapshot.signed_entitlement.as_ref() else {
            snapshot.state = LicenseState::Unavailable;
            return;
        };
        if let Err(error) = Self::verify_signed_entitlement(&self.config, snapshot) {
            snapshot.state = LicenseState::Invalid;
            snapshot.last_error = Some(error);
            return;
        }
        let now = Utc::now();
        if now + Duration::minutes(5) < snapshot.last_known_time {
            snapshot.state = LicenseState::Invalid;
            snapshot.last_error = Some("The system clock moved backwards.".into());
            return;
        }
        if snapshot
            .server_status
            .as_deref()
            .is_some_and(|status| status.eq_ignore_ascii_case("suspended"))
        {
            snapshot.state = LicenseState::Suspended;
            return;
        }
        if snapshot
            .server_status
            .as_deref()
            .is_some_and(|status| status.eq_ignore_ascii_case("revoked"))
        {
            snapshot.state = LicenseState::Revoked;
            return;
        }
        if snapshot.server_status.as_deref().is_some_and(|status| {
            !status.eq_ignore_ascii_case("active")
                && !status.eq_ignore_ascii_case("suspended")
                && !status.eq_ignore_ascii_case("revoked")
        }) {
            snapshot.state = LicenseState::Invalid;
            snapshot.last_error = Some("Mulverse returned a non-active license status.".into());
            return;
        }
        let payload = &entitlement.payload;
        if now < payload.valid_from {
            snapshot.state = LicenseState::Pending;
        } else if payload
            .valid_until
            .is_some_and(|valid_until| valid_until <= now)
        {
            snapshot.state = LicenseState::Expired;
        } else {
            let interval = payload
                .verification_interval_hours
                .filter(|hours| *hours > 0)
                .map(i64::from)
                .unwrap_or(DEFAULT_VERIFY_INTERVAL_HOURS);
            let grace_days = payload.offline_grace_period_days.unwrap_or_default().max(0);
            let grace_end = payload.issued_at + Duration::days(i64::from(grace_days));
            if now > grace_end {
                if !payload.offline_allowed {
                    snapshot.state = LicenseState::Invalid;
                } else {
                    snapshot.state = LicenseState::Expired;
                }
            } else if payload.valid_until.is_some_and(|valid_until| {
                valid_until - now <= Duration::days(EXPIRING_WITHIN_DAYS)
            }) {
                snapshot.state = LicenseState::Expiring;
            } else if now > payload.issued_at + Duration::hours(interval) {
                snapshot.state = LicenseState::OfflineGrace;
            } else if is_trial(payload.plan_key.as_deref()) {
                snapshot.state = LicenseState::Trial;
            } else {
                snapshot.state = LicenseState::Active;
            }
        }
        snapshot.last_known_time = now;
    }

    fn authorize_snapshot(
        &self,
        snapshot: &LicenseSnapshot,
        organization_id: &str,
        feature: Option<&str>,
    ) -> Result<(), String> {
        if !self.config.is_configured() {
            return Err(self
                .config
                .error
                .clone()
                .unwrap_or_else(|| "Mulverse licensing is not configured.".into()));
        }
        let mut snapshot = snapshot.clone();
        self.evaluate(&mut snapshot);
        if !snapshot.state.is_operational() {
            return Err(match snapshot.last_error {
                Some(error) => error,
                None => format!(
                    "A valid AqarDesk license is required ({}).",
                    state_label(&snapshot.state)
                ),
            });
        }
        let entitlement = snapshot
            .signed_entitlement
            .as_ref()
            .ok_or_else(|| "A valid AqarDesk license is required.".to_string())?;
        if !same_uuid(&entitlement.payload.organization_id, organization_id) {
            return Err("The AqarDesk license does not match the active organization.".into());
        }
        if let Some(feature) = feature {
            let granted = entitlement
                .payload
                .entitlements
                .iter()
                .any(|entry| entry.key == feature && entry.granted);
            if !granted {
                return Err(format!(
                    "The active plan does not include the {feature} feature."
                ));
            }
        }
        Ok(())
    }

    fn ensure_configured(&self) -> Result<(), String> {
        self.config.error.clone().map_or(Ok(()), Err)
    }

    async fn store(&self, snapshot: LicenseSnapshot) -> Result<(), String> {
        let path = self.app_data_dir.join("license-state.json");
        let temporary = self.app_data_dir.join("license-state.json.tmp");
        let bytes = serde_json::to_vec(&snapshot).map_err(|error| {
            tracing::error!(%error, "Could not serialize license state");
            "Could not save licensing information.".to_string()
        })?;
        std::fs::write(&temporary, bytes).map_err(|error| {
            tracing::error!(%error, "Could not write license state");
            "Could not save licensing information.".to_string()
        })?;
        if path.exists() {
            std::fs::remove_file(&path).map_err(|error| {
                tracing::error!(%error, "Could not replace the saved license state");
                "Could not update licensing information.".to_string()
            })?;
        }
        std::fs::rename(&temporary, &path).map_err(|error| {
            tracing::error!(%error, "Could not finalize the saved license state");
            "Could not update licensing information.".to_string()
        })?;
        *self.snapshot.write().await = snapshot;
        Ok(())
    }
}

fn new_installation_id() -> String {
    format!("inst_{}_{}", std::env::consts::OS, Uuid::new_v4().simple())
}

fn load_or_create_installation_id(app_data_dir: &std::path::Path) -> Result<String, String> {
    let path = app_data_dir.join("license-installation-id");
    match std::fs::read_to_string(&path) {
        Ok(value) if !value.trim().is_empty() && value.trim().len() <= 128 => {
            Ok(value.trim().to_string())
        }
        Ok(_) => Err("The saved AqarDesk installation identifier is invalid.".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let installation_id = new_installation_id();
            std::fs::write(&path, &installation_id).map_err(|error| {
                tracing::error!(%error, "Could not persist AqarDesk installation identity");
                "Could not save this installation's licensing identity.".to_string()
            })?;
            Ok(installation_id)
        }
        Err(error) => {
            tracing::error!(%error, "Could not read AqarDesk installation identity");
            Err("Could not read this installation's licensing identity.".into())
        }
    }
}

fn is_trial(plan_key: Option<&str>) -> bool {
    plan_key.is_some_and(|plan| plan.eq_ignore_ascii_case("TRIAL"))
}

fn application_key_requires_claim(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    (message.contains("not assigned to an customer")
        || message.contains("not assigned to a customer"))
        && message.contains("claim it or assign it before activation")
}

fn customer_slug(organization_id: Uuid, organization_name: &str) -> String {
    let organization_suffix = organization_id.simple().to_string();
    format!(
        "{}-{}",
        slugify(organization_name)
            .chars()
            .take(48)
            .collect::<String>(),
        &organization_suffix[..8]
    )
}

fn same_uuid(left: &str, right: &str) -> bool {
    match (Uuid::parse_str(left), Uuid::parse_str(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn slugify(value: &str) -> String {
    let slug: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else if character.is_whitespace() || character == '-' || character == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect();
    let slug = slug
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "aqardesk-customer".into()
    } else {
        slug.chars().take(64).collect()
    }
}

fn limit_unit(key: &str) -> &'static str {
    LIMIT_KEYS
        .iter()
        .find_map(|(known_key, unit)| (*known_key == key).then_some(*unit))
        .unwrap_or("")
}

fn state_label(state: &LicenseState) -> &'static str {
    match state {
        LicenseState::Unavailable => "not activated",
        LicenseState::Pending => "pending",
        LicenseState::Trial => "trial",
        LicenseState::Active => "active",
        LicenseState::Expiring => "expiring",
        LicenseState::OfflineGrace => "offline grace period",
        LicenseState::DeactivationPending => "deactivation pending",
        LicenseState::Expired => "expired",
        LicenseState::Suspended => "suspended",
        LicenseState::Revoked => "revoked",
        LicenseState::Invalid => "invalid",
    }
}

impl LicenseRuntime {
    async fn sync_interval_seconds(&self) -> u64 {
        let snapshot = self.snapshot.read().await;
        snapshot
            .signed_entitlement
            .as_ref()
            .and_then(|entitlement| entitlement.payload.verification_interval_hours)
            .filter(|hours| *hours > 0)
            .map(|hours| u64::try_from(hours).unwrap_or(DEFAULT_VERIFY_INTERVAL_HOURS as u64))
            .unwrap_or(DEFAULT_VERIFY_INTERVAL_HOURS as u64)
            .saturating_mul(60 * 60)
    }
}

pub fn spawn_background_sync(runtime: Arc<LicenseRuntime>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(StdDuration::from_secs(60)).await;
        loop {
            match runtime.refresh(None).await {
                Ok(_) => tracing::debug!("Background Mulverse license sync completed"),
                Err(error) => tracing::debug!(%error, "Background Mulverse license sync failed"),
            }
            tokio::time::sleep(StdDuration::from_secs(
                runtime.sync_interval_seconds().await,
            ))
            .await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signed_runtime() -> (LicenseRuntime, String) {
        let signing_key = SigningKey::from_bytes(&[19_u8; 32]);
        let organization_id = Uuid::new_v4().to_string();
        let installation_id = "inst_test_123".to_string();
        let product_id = "53c4f4f4-7cb7-4e31-8fd1-1bca367ea8d5".to_string();
        let payload = EntitlementPayload {
            license_id: "license-test".into(),
            product_key: "AQD-ABCDE-FGHIJ".into(),
            product_id: product_id.clone(),
            organization_id: organization_id.clone(),
            edition_key: None,
            plan_key: Some("PRO".into()),
            plan_name: Some("Pro".into()),
            entitlements: vec![EntitlementItem {
                key: "properties".into(),
                granted: true,
            }],
            limits: vec![LimitItem {
                key: "properties.max".into(),
                max_value: Some(5),
            }],
            activation_id: "activation-test".into(),
            installation_id: installation_id.clone(),
            issued_at: Utc::now(),
            valid_from: Utc::now() - Duration::minutes(1),
            valid_until: Some(Utc::now() + Duration::days(30)),
            offline_allowed: true,
            offline_grace_period_days: Some(30),
            verification_interval_hours: Some(24),
        };
        let signature = signing_key.sign(&serde_json::to_vec(&payload).unwrap());
        let entitlement = SignedEntitlement {
            v: 1,
            kid: "test-key".into(),
            alg: "Ed25519".into(),
            payload,
            sig: STANDARD.encode(signature.to_bytes()),
        };
        let config = LicenseConfig {
            api_base_url: DEFAULT_API_URL.into(),
            product_code: "AQARDESK".into(),
            product_id,
            key_prefix: "AQD-".into(),
            public_key_id: "test-key".into(),
            public_key_b64: STANDARD.encode(signing_key.verifying_key().to_bytes()),
            error: None,
        };
        let snapshot = LicenseSnapshot {
            state: LicenseState::Active,
            signed_entitlement: Some(entitlement),
            installation_id,
            app_version: "test".into(),
            last_sync_at: Some(Utc::now()),
            last_known_time: Utc::now(),
            server_status: Some("active".into()),
            last_error: None,
        };
        (
            LicenseRuntime {
                config,
                app_data_dir: std::env::temp_dir().join(Uuid::new_v4().to_string()),
                client: reqwest::Client::new(),
                snapshot: RwLock::new(snapshot),
                lifecycle_lock: Mutex::new(()),
            },
            organization_id,
        )
    }

    #[test]
    fn signed_entitlement_is_bound_to_product_and_installation() {
        let (runtime, _) = signed_runtime();
        let snapshot = runtime.snapshot.blocking_read().clone();
        assert!(LicenseRuntime::verify_signed_entitlement(&runtime.config, &snapshot).is_ok());

        let mut tampered = snapshot.clone();
        tampered.signed_entitlement.as_mut().unwrap().payload.limits[0].max_value = Some(500);
        assert!(LicenseRuntime::verify_signed_entitlement(&runtime.config, &tampered).is_err());

        let mut wrong_installation = snapshot;
        wrong_installation.installation_id = "different-installation".into();
        assert!(
            LicenseRuntime::verify_signed_entitlement(&runtime.config, &wrong_installation)
                .is_err()
        );
    }

    #[test]
    fn entitlement_configuration_mismatches_identify_the_setting_to_check() {
        let (runtime, _) = signed_runtime();
        let snapshot = runtime.snapshot.blocking_read().clone();

        let mut wrong_key = snapshot.clone();
        wrong_key.signed_entitlement.as_mut().unwrap().kid = "other-key".into();
        assert_eq!(
            LicenseRuntime::verify_signed_entitlement(&runtime.config, &wrong_key).unwrap_err(),
            "Mulverse signed the license with a different key ID. Check MULVERSE_LICENSE_PUBLIC_KEY_ID."
        );

        let mut wrong_product = snapshot.clone();
        wrong_product
            .signed_entitlement
            .as_mut()
            .unwrap()
            .payload
            .product_id = Uuid::new_v4().to_string();
        assert_eq!(
            LicenseRuntime::verify_signed_entitlement(&runtime.config, &wrong_product).unwrap_err(),
            "Mulverse returned a license for a different product. Check MULVERSE_LICENSE_PRODUCT_ID."
        );

        let mut wrong_installation = snapshot;
        wrong_installation
            .signed_entitlement
            .as_mut()
            .unwrap()
            .payload
            .installation_id = "another-installation".into();
        assert_eq!(
            LicenseRuntime::verify_signed_entitlement(&runtime.config, &wrong_installation)
                .unwrap_err(),
            "Mulverse returned a license for a different installation. Retry from the same AqarDesk installation."
        );
    }

    #[tokio::test]
    async fn resource_limits_reject_the_first_item_over_the_signed_maximum() {
        let (runtime, organization_id) = signed_runtime();
        assert!(
            runtime
                .check_limit_for_organization(&organization_id, "properties.max", 4, 1)
                .await
                .is_ok()
        );
        assert!(
            runtime
                .check_limit_for_organization(&organization_id, "properties.max", 5, 1)
                .await
                .is_err()
        );
        assert!(
            runtime
                .check_limit_for_organization(&Uuid::new_v4().to_string(), "properties.max", 0, 1)
                .await
                .is_err()
        );
    }

    #[test]
    fn product_key_validation_uses_aqar_desk_prefix() {
        let (runtime, _) = signed_runtime();
        assert!(
            runtime
                .validate_product_key("AQD-ABCDE-FGHIJ-KLMNO-PQRST")
                .is_ok()
        );
        assert!(
            runtime
                .validate_product_key("OTHER-ABCDE-FGHIJ-KLMNO-PQRST")
                .is_err()
        );
        assert!(runtime.validate_product_key("AQD-").is_err());
    }

    #[test]
    fn reseller_keys_are_claimed_only_for_the_expected_unassigned_key_error() {
        assert!(application_key_requires_claim(
            "This application key is not assigned to an customer. Claim it or assign it before activation."
        ));
        assert!(!application_key_requires_claim(
            "The license is not assigned to the active customer."
        ));
        assert!(!application_key_requires_claim(
            "The product key was not found."
        ));
    }

    #[test]
    fn missing_product_configuration_fails_closed() {
        let (mut runtime, organization_id) = signed_runtime();
        runtime.config.error = Some("Mulverse product identity is not configured.".into());
        let snapshot = runtime.snapshot.try_read().unwrap().clone();
        assert!(
            runtime
                .authorize_snapshot(&snapshot, &organization_id, None)
                .is_err()
        );
    }
}
