use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeygenConfig {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKey {
    pub key: String,
    pub user_email: Option<String>,
    pub cached_validation: Option<LicenseValidation>,
    pub last_validated: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseValidation {
    pub valid: bool,
    pub license_type: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub policy_name: Option<String>,
    pub usage_count: Option<u64>,
    pub usage_limit: Option<u64>,
}

#[derive(Debug, Serialize)]
struct KeygenValidationRequest {
    meta: KeygenMeta,
}

#[derive(Debug, Serialize)]
struct KeygenMeta {
    key: String,
    scope: KeygenScope,
}

#[derive(Debug, Serialize)]
struct KeygenScope {
    fingerprint: String,
}

#[derive(Debug, Serialize)]
struct MachineActivationRequest {
    data: MachineActivationData,
}

#[derive(Debug, Serialize)]
struct MachineActivationData {
    #[serde(rename = "type")]
    data_type: String,
    attributes: MachineAttributes,
    relationships: MachineRelationships,
}

#[derive(Debug, Serialize)]
struct MachineAttributes {
    fingerprint: String,
    name: String,
    platform: String,
    hostname: String,
    cores: u32,
}

#[derive(Debug, Serialize)]
struct MachineRelationships {
    license: MachineRelationshipData,
}

#[derive(Debug, Serialize)]
struct MachineRelationshipData {
    data: MachineRelationshipItem,
}

#[derive(Debug, Serialize)]
struct MachineRelationshipItem {
    #[serde(rename = "type")]
    item_type: String,
    id: String,
}

#[derive(Debug, Deserialize)]
struct KeygenResponse {
    data: Option<KeygenLicenseData>,
    meta: Option<KeygenValidationMeta>,
    errors: Option<Vec<KeygenError>>,
}

#[derive(Debug, Deserialize)]
struct KeygenValidationMeta {
    valid: bool,
    detail: String,
    code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KeygenLicenseData {
    id: String,
    #[serde(rename = "type")]
    data_type: String,
    attributes: KeygenLicenseAttributes,
    relationships: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct KeygenLicenseAttributes {
    name: Option<String>,
    key: String,
    expiry: Option<String>,
    status: String,
    uses: Option<u64>,
    #[serde(rename = "maxUses")]
    max_uses: Option<u64>,
    suspended: bool,
    floating: bool,
    #[serde(rename = "lastValidated")]
    last_validated: Option<String>,
    created: String,
    updated: String,
}

#[derive(Debug, Deserialize)]
struct KeygenError {
    title: String,
    detail: String,
    code: String,
}

pub struct LicenseManager {
    config: KeygenConfig,
    client: reqwest::Client,
    license_file_path: String,
}

impl LicenseManager {
    pub fn new() -> Result<Self> {
        let config = Self::load_keygen_config()?;
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        
        let license_file_path = Self::get_license_file_path()?;
        
        Ok(Self {
            config,
            client,
            license_file_path,
        })
    }

    fn generate_machine_fingerprint() -> Result<String> {
        use sha2::{Sha256, Digest};

        // Get machine GUID (most reliable identifier)
        let machine_guid = Self::get_machine_guid()?;

        // Create SHA256 hash of the machine GUID for anonymization
        let mut hasher = Sha256::new();
        hasher.update(machine_guid.as_bytes());
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    fn get_machine_guid() -> Result<String> {
        #[cfg(target_os = "windows")]
        {
            // Try to get Windows machine GUID from WMI
            if let Ok(output) = std::process::Command::new("powershell")
                .args(&["-c", "Get-WmiObject -Class Win32_ComputerSystemProduct | Select-Object -ExpandProperty UUID"])
                .output()
            {
                if output.status.success() {
                    let guid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !guid.is_empty() && guid != "FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF" {
                        return Ok(guid);
                    }
                }
            }

            // Fallback to hostname + processor info
            let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
            let processor = std::env::var("PROCESSOR_IDENTIFIER").unwrap_or_else(|_| "unknown".to_string());
            Ok(format!("{}-{}", hostname, processor))
        }

        #[cfg(target_os = "macos")]
        {
            // Try to get macOS machine GUID
            if let Ok(output) = std::process::Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                if output.status.success() {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Parse IOPlatformUUID from the output
                    for line in output_str.lines() {
                        if line.contains("IOPlatformUUID") {
                            if let Some(uuid) = line.split('"').nth(3) {
                                return Ok(uuid.to_string());
                            }
                        }
                    }
                }
            }

            // Fallback
            let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
            Ok(hostname)
        }

        #[cfg(target_os = "linux")]
        {
            // Try to get Linux machine ID
            if let Ok(machine_id) = std::fs::read_to_string("/etc/machine-id") {
                return Ok(machine_id.trim().to_string());
            } else if let Ok(machine_id) = std::fs::read_to_string("/var/lib/dbus/machine-id") {
                return Ok(machine_id.trim().to_string());
            }

            // Fallback
            let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
            Ok(hostname)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // Generic fallback for other platforms
            let hostname = std::env::var("HOSTNAME")
                .or_else(|_| std::env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "unknown".to_string());
            Ok(hostname)
        }
    }

    fn get_system_info() -> (String, String, u32) {
        let platform = if cfg!(target_os = "windows") {
            "win32".to_string()
        } else if cfg!(target_os = "macos") {
            "darwin".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "unknown".to_string()
        };

        let hostname = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        // Try to get CPU core count
        let cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);

        (platform, hostname, cores)
    }

    fn load_keygen_config() -> Result<KeygenConfig> {
        // Try to load from .env.keygen file first
        let env_keygen_path = ".env.keygen";
        if Path::new(env_keygen_path).exists() {
            let content = fs::read_to_string(env_keygen_path)?;
            let mut config = KeygenConfig {
                account_id: String::new(),
            };

            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    match key.trim() {
                        "KEYGEN_ACCOUNT_ID" => config.account_id = value.trim().to_string(),
                        _ => {}
                    }
                }
            }

            if config.account_id.is_empty() {
                return Err(anyhow!("KEYGEN_ACCOUNT_ID missing in .env.keygen"));
            }

            return Ok(config);
        }

        // Try to load from .env file
        let env_path = ".env";
        if Path::new(env_path).exists() {
            let content = fs::read_to_string(env_path)?;
            let mut config = KeygenConfig {
                account_id: String::new(),
            };

            for line in content.lines() {
                if let Some((key, value)) = line.split_once('=') {
                    match key.trim() {
                        "KEYGEN_ACCOUNT_ID" => config.account_id = value.trim().to_string(),
                        _ => {}
                    }
                }
            }

            if !config.account_id.is_empty() {
                return Ok(config);
            }
        }
        
        // Fallback to environment variables
        let account_id = std::env::var("KEYGEN_ACCOUNT_ID")
            .map_err(|_| anyhow!("KEYGEN_ACCOUNT_ID not found in environment or .env file"))?;

        Ok(KeygenConfig {
            account_id,
        })
    }

    fn get_license_file_path() -> Result<String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let license_dir = home_dir.join(".aimemoryengine");
        
        if !license_dir.exists() {
            fs::create_dir_all(&license_dir)?;
        }
        
        Ok(license_dir.join("license.json").to_string_lossy().to_string())
    }

    async fn activate_machine(&self, license_key: &str, license_id: &str) -> Result<()> {
        let fingerprint = Self::generate_machine_fingerprint()?;
        let (platform, hostname, cores) = Self::get_system_info();

        let url = format!("https://api.keygen.sh/v1/accounts/{}/machines",
                         self.config.account_id);

        let machine_request = MachineActivationRequest {
            data: MachineActivationData {
                data_type: "machines".to_string(),
                attributes: MachineAttributes {
                    fingerprint: fingerprint.clone(),
                    name: format!("AI Memory Engine - {}", hostname),
                    platform,
                    hostname,
                    cores,
                },
                relationships: MachineRelationships {
                    license: MachineRelationshipData {
                        data: MachineRelationshipItem {
                            item_type: "licenses".to_string(),
                            id: license_id.to_string(),
                        },
                    },
                },
            },
        };

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/vnd.api+json")
            .header("Accept", "application/vnd.api+json")
            .header("Authorization", format!("License {}", license_key))
            .json(&machine_request)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("Machine activation failed: {} - {}", status, response_text));
        }

        println!("✅ Machine activated successfully!");
        Ok(())
    }

    pub async fn validate_license(&self, license_key: &str) -> Result<LicenseValidation> {
        self.validate_license_internal(license_key, false).await
    }

    async fn validate_license_internal(&self, license_key: &str, retry_after_activation: bool) -> Result<LicenseValidation> {
        let url = format!("https://api.keygen.sh/v1/accounts/{}/licenses/actions/validate-key",
                         self.config.account_id);

        let fingerprint = Self::generate_machine_fingerprint()?;

        let request_body = KeygenValidationRequest {
            meta: KeygenMeta {
                key: license_key.to_string(),
                scope: KeygenScope {
                    fingerprint: fingerprint.clone(),
                },
            },
        };

        // Use NO AUTHENTICATION for license validation - this is public endpoint
        let response = self.client
            .post(&url)
            .header("Content-Type", "application/vnd.api+json")
            .header("Accept", "application/vnd.api+json")
            .json(&request_body)
            .send()
            .await?;

        let _status = response.status();
        let response_text = response.text().await?;

        // Debug logging (remove in production)
        // println!("DEBUG: HTTP Status: {}", _status);
        // println!("DEBUG: Response: {}", response_text);
        // println!("DEBUG: Fingerprint used: {}", fingerprint);

        let keygen_response: KeygenResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Keygen response: {} - Response: {}", e, response_text))?;

        // Check for API errors first
        if let Some(errors) = keygen_response.errors {
            let error_msg = errors.iter()
                .map(|e| format!("{}: {}", e.title, e.detail))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow!("License validation failed: {}", error_msg));
        }

        // Get validation result from meta
        let validation_meta = keygen_response.meta
            .ok_or_else(|| anyhow!("No validation meta in response"))?;

        let mut license_validation = LicenseValidation {
            valid: validation_meta.valid,
            license_type: "professional".to_string(),
            expires_at: None,
            user_name: None,
            user_email: None,
            policy_name: None,
            usage_count: None,
            usage_limit: None,
        };

        // If validation was successful and we have license data, extract additional info
        if validation_meta.valid {
            if let Some(license_data) = keygen_response.data {
                // Check if license is suspended or has other issues
                if license_data.attributes.suspended {
                    license_validation.valid = false;
                }

                // Parse expiry date
                license_validation.expires_at = license_data.attributes.expiry
                    .and_then(|exp| chrono::DateTime::parse_from_rfc3339(&exp).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc));

                // Set usage information
                license_validation.usage_count = license_data.attributes.uses;
                license_validation.usage_limit = license_data.attributes.max_uses;

                // Set license name if available
                if let Some(name) = license_data.attributes.name {
                    license_validation.policy_name = Some(name);
                }
            }
        } else if !retry_after_activation {
            // Check if validation failed due to missing machine (NO_MACHINE error)
            if let Some(ref code) = validation_meta.code {
                if code == "NO_MACHINE" {
                    println!("🔧 License requires machine activation. Activating this machine...");

                    // Extract license ID from response data
                    if let Some(license_data) = keygen_response.data {
                        let license_id = license_data.id;

                        // Try to activate machine
                        if let Err(e) = self.activate_machine(license_key, &license_id).await {
                            return Err(anyhow!("Failed to activate machine: {}", e));
                        }

                        // Retry validation after machine activation (with flag to prevent infinite recursion)
                        println!("🔄 Retrying license validation...");
                        return Box::pin(self.validate_license_internal(license_key, true)).await;
                    } else {
                        return Err(anyhow!("License validation failed: {}", validation_meta.detail));
                    }
                }
            }
        }

        Ok(license_validation)
    }

    pub fn save_license(&self, license_key: &str, validation: &LicenseValidation) -> Result<()> {
        let license = LicenseKey {
            key: license_key.to_string(),
            user_email: validation.user_email.clone(),
            cached_validation: Some(validation.clone()),
            last_validated: Some(chrono::Utc::now()),
        };
        
        let license_json = serde_json::to_string_pretty(&license)?;
        fs::write(&self.license_file_path, license_json)?;
        
        Ok(())
    }

    pub fn load_cached_license(&self) -> Result<Option<LicenseKey>> {
        if !Path::new(&self.license_file_path).exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&self.license_file_path)?;
        let license: LicenseKey = serde_json::from_str(&content)?;
        
        Ok(Some(license))
    }

    pub fn is_cache_valid(&self, license: &LicenseKey) -> bool {
        if let Some(last_validated) = license.last_validated {
            let cache_duration = chrono::Duration::hours(24); // Cache for 24 hours
            chrono::Utc::now() - last_validated < cache_duration
        } else {
            false
        }
    }

    pub async fn check_license(&self, license_key: Option<&str>) -> Result<LicenseValidation> {
        // Try to load cached license first
        if let Ok(Some(cached_license)) = self.load_cached_license() {
            if license_key.is_none() || license_key == Some(&cached_license.key) {
                if self.is_cache_valid(&cached_license) {
                    if let Some(validation) = cached_license.cached_validation {
                        return Ok(validation);
                    }
                }
            }
        }

        // If no cached license or cache expired, validate online
        let key_to_validate = if let Some(key) = license_key {
            key.to_string()
        } else if let Ok(Some(cached)) = self.load_cached_license() {
            cached.key
        } else {
            return Err(anyhow!("No license key provided and no cached license found"));
        };

        let validation = self.validate_license(&key_to_validate).await?;

        // Cache the validation result
        if validation.valid {
            self.save_license(&key_to_validate, &validation)?;
        }

        Ok(validation)
    }

    pub fn remove_license(&self) -> Result<()> {
        if Path::new(&self.license_file_path).exists() {
            fs::remove_file(&self.license_file_path)?;
        }
        Ok(())
    }
}
