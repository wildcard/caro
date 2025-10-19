# Telemetry & Privacy Framework

## Overview

**Document Type**: Privacy and Data Design  
**Related PRD**: Command Generation Enhancement  
**Version**: 1.0  
**Date**: 2025-10-19  
**Compliance**: GDPR, CCPA, Privacy-by-Design  

### Purpose
This document defines the privacy-first telemetry framework for cmdai that enables continuous improvement through anonymous usage analytics while maintaining strict user privacy protection and regulatory compliance.

## Privacy Philosophy

### Core Principles
1. **Privacy by Design**: Privacy protection built into system architecture
2. **Data Minimization**: Collect only essential data for improvement
3. **User Control**: Users have complete control over data sharing
4. **Transparency**: Clear, understandable privacy practices
5. **Anonymous by Default**: No personal information collection
6. **Local Processing**: All sensitive processing happens locally
7. **Opt-in Only**: No data collection without explicit consent

### Privacy Statement
> "cmdai collects only anonymous, aggregated usage data to improve command accuracy. We never collect personal information, file paths, or sensitive data. All data collection is optional and can be disabled at any time."

## Data Collection Framework

### What We Collect (Anonymous Only)

#### Session Data
```rust
pub struct AnonymousSession {
    pub session_id: Uuid,           // Random UUID per session
    pub timestamp: DateTime<Utc>,   // UTC timestamp
    pub shell_type: ShellType,      // bash, zsh, fish, etc.
    pub os_type: String,            // linux, macos, windows
    pub cmdai_version: String,      // cmdai version string
    pub locale: Option<String>,     // Language preference (optional)
    pub country_code: Option<String>, // Coarse location (country only)
    // NO user identification, usernames, or personal data
}
```

#### Command Generation Data
```rust
pub struct AnonymousCommandEvent {
    pub event_id: Uuid,             // Random event identifier
    pub session_id: Uuid,           // Links to session
    pub input_hash: String,         // SHA-256 hash of natural language input
    pub input_length: u32,          // Length of input (for analysis)
    pub input_language: Option<String>, // Detected language (optional)
    pub generated_command: String,  // Generated shell command
    pub alternatives_count: u32,    // Number of alternatives provided
    pub generation_time_ms: u64,    // Time to generate command
    pub safety_level: SafetyLevel,  // Assessed safety level
    pub backend_used: String,       // Which backend generated command
    pub timestamp: DateTime<Utc>,   // Event timestamp
    // NO original input text, file names, or user content
}
```

#### User Feedback Data
```rust
pub struct AnonymousFeedback {
    pub feedback_id: Uuid,          // Random feedback identifier
    pub event_id: Uuid,             // Links to command event
    pub user_action: UserAction,    // Accepted, rejected, modified, cancelled
    pub execution_result: Option<ExecutionResult>, // Success, failure, timeout
    pub modified_command: Option<String>, // If user modified the command
    pub satisfaction_rating: Option<u8>, // 1-5 rating (optional)
    pub feedback_timestamp: DateTime<Utc>, // When feedback was given
    // NO personal opinions, comments, or identifying information
}

pub enum UserAction {
    Accepted,           // User ran the command as-is
    Rejected,           // User didn't run the command
    Modified,           // User modified before running
    Cancelled,          // User cancelled operation
    Alternative,        // User chose an alternative
}

pub enum ExecutionResult {
    Success,            // Command executed successfully
    Failure,            // Command failed
    Timeout,            // Command timed out
    Cancelled,          // User cancelled execution
}
```

### What We NEVER Collect

#### Explicitly Prohibited Data
- **Personal Information**: Names, emails, usernames, UIDs
- **File Paths**: Working directories, file names, folder structures
- **File Content**: Contents of files, command output, error messages
- **Network Information**: IP addresses, MAC addresses, hostnames
- **Sensitive Commands**: Commands containing passwords, keys, tokens
- **Location Data**: Precise location beyond country-level
- **Device Information**: Serial numbers, hardware specifics, unique identifiers
- **Browsing Data**: URLs, browser information, other applications
- **System Configuration**: Environment variables, config files, installed software lists

### Data Anonymization Process

#### Input Sanitization
```rust
pub struct InputSanitizer {
    pub fn sanitize_input(&self, input: &str) -> SanitizedInput {
        let hash = self.create_hash(input);
        let length = input.len() as u32;
        let language = self.detect_language(input);
        
        // Remove any potential personal information
        SanitizedInput {
            hash,
            length,
            language,
            // Original input is discarded
        }
    }
    
    fn create_hash(&self, input: &str) -> String {
        // SHA-256 hash with salt to prevent rainbow table attacks
        let salt = "cmdai-telemetry-salt-v1";
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
```

#### Command Filtering
```rust
pub struct CommandFilter {
    pub fn filter_sensitive_commands(&self, command: &str) -> Option<String> {
        // Check for sensitive patterns
        if self.contains_sensitive_data(command) {
            return None; // Don't collect sensitive commands
        }
        
        // Remove potential personal data from commands
        let filtered = self.remove_personal_data(command);
        Some(filtered)
    }
    
    fn contains_sensitive_data(&self, command: &str) -> bool {
        let sensitive_patterns = [
            r"password",
            r"token",
            r"key",
            r"secret",
            r"auth",
            r"ssh",
            r"gpg",
            r"curl.*-u",
            r"wget.*--user",
            // Add more patterns
        ];
        
        sensitive_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(command)
        })
    }
}
```

## User Consent Framework

### Opt-in Process
```rust
pub struct ConsentManager {
    pub fn request_consent() -> ConsentResult {
        println!("📊 Help improve cmdai");
        println!("Would you like to contribute anonymous usage data to help improve");
        println!("command generation accuracy? This helps us understand common");
        println!("usage patterns and fix issues.");
        println!();
        println!("We collect:");
        println!("  ✓ Anonymous command patterns (hashed inputs)");
        println!("  ✓ Generated commands and success rates");
        println!("  ✓ Performance metrics and error patterns");
        println!();
        println!("We NEVER collect:");
        println!("  ✗ Personal information or usernames");
        println!("  ✗ File paths or directory structures");
        println!("  ✗ Command output or file contents");
        println!("  ✗ Sensitive data or credentials");
        println!();
        println!("You can change this setting anytime with 'cmdai --configure'");
        println!();
        
        let consent = prompt_user_choice(&[
            "Yes, contribute anonymous data",
            "No, keep everything local",
            "Show detailed privacy policy"
        ]);
        
        match consent {
            0 => ConsentResult::Granted,
            1 => ConsentResult::Denied,
            2 => {
                self.show_detailed_privacy_policy();
                self.request_consent() // Ask again after showing policy
            }
            _ => ConsentResult::Denied,
        }
    }
}
```

### Consent Management
```rust
pub struct TelemetryConfig {
    pub enabled: bool,              // Master toggle
    pub command_patterns: bool,     // Command generation data
    pub performance_metrics: bool,  // Performance and error data
    pub feedback_data: bool,        // User feedback and satisfaction
    pub frequency: UpdateFrequency, // How often to send data
    pub retention_days: u32,        // How long to keep local data
}

pub enum UpdateFrequency {
    Immediate,      // Send data immediately
    Daily,          // Batch daily
    Weekly,         // Batch weekly
    Manual,         // User-triggered only
}
```

### Privacy Controls
```bash
# Configuration interface
cmdai --configure
> Privacy & Analytics
  > Telemetry: Enabled
  > Data Types: Command patterns, Performance metrics
  > Frequency: Daily batches
  > Local retention: 30 days
  > Actions:
    - View collected data
    - Export my data
    - Delete all data
    - Disable telemetry
    - Show privacy policy
```

## Data Transmission and Storage

### Transmission Security
```rust
pub struct SecureTransmission {
    pub fn send_telemetry_batch(&self, data: TelemetryBatch) -> Result<(), TelemetryError> {
        // Encrypt data before transmission
        let encrypted_data = self.encrypt_data(&data)?;
        
        // Send via HTTPS with certificate pinning
        let response = self.https_client
            .post("https://analytics.cmdai.dev/v1/telemetry")
            .header("Content-Type", "application/octet-stream")
            .header("User-Agent", format!("cmdai/{}", VERSION))
            .body(encrypted_data)
            .send()?;
            
        // Verify successful transmission
        if response.status().is_success() {
            self.clear_local_batch()?;
            Ok(())
        } else {
            Err(TelemetryError::TransmissionFailed)
        }
    }
    
    fn encrypt_data(&self, data: &TelemetryBatch) -> Result<Vec<u8>, CryptoError> {
        // Use ChaCha20Poly1305 for authenticated encryption
        let key = self.get_ephemeral_key();
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        
        let serialized = bincode::serialize(data)?;
        let encrypted = cipher.encrypt(&nonce, serialized.as_ref())?;
        
        Ok([nonce.as_slice(), &encrypted].concat())
    }
}
```

### Analytics Service Architecture
```rust
// Separate service: cmdai-analytics
pub struct AnalyticsService {
    // Stateless processing service
    // No persistent user data storage
    // Aggregate statistics only
    
    pub fn process_telemetry_batch(&self, batch: EncryptedBatch) -> ProcessingResult {
        // Decrypt and validate batch
        let decrypted = self.decrypt_batch(batch)?;
        
        // Process for aggregate statistics
        let aggregated = self.aggregate_data(decrypted);
        
        // Update statistical models
        self.update_models(aggregated)?;
        
        // Discard individual data points
        // Only keep aggregated statistics
        ProcessingResult::Success
    }
}
```

### Local Data Management
```rust
pub struct LocalTelemetryStorage {
    pub fn store_event(&self, event: AnonymousEvent) -> Result<(), StorageError> {
        // Store in local SQLite database with encryption
        let encrypted_event = self.encrypt_for_storage(event)?;
        self.db.execute(
            "INSERT INTO telemetry_events (id, data, timestamp) VALUES (?, ?, ?)",
            params![encrypted_event.id, encrypted_event.data, encrypted_event.timestamp]
        )?;
        
        // Automatic cleanup of old data
        self.cleanup_old_data()?;
        Ok(())
    }
    
    fn cleanup_old_data(&self) -> Result<(), StorageError> {
        let retention_date = Utc::now() - Duration::days(30);
        self.db.execute(
            "DELETE FROM telemetry_events WHERE timestamp < ?",
            params![retention_date]
        )?;
        Ok(())
    }
}
```

## Compliance and Regulations

### GDPR Compliance

#### Data Subject Rights
1. **Right to Information**: Clear privacy policy and data usage explanation
2. **Right of Access**: Users can view all collected data
3. **Right to Rectification**: Ability to correct inaccurate data
4. **Right to Erasure**: Complete data deletion on request
5. **Right to Portability**: Export collected data in machine-readable format
6. **Right to Object**: Opt-out of data collection at any time

#### Implementation
```rust
pub struct GDPRCompliance {
    pub fn export_user_data(&self, user_id: Option<String>) -> Result<DataExport, ComplianceError> {
        // Since we don't store user IDs, export based on local device data
        let local_data = self.get_local_telemetry_data()?;
        Ok(DataExport {
            format: "JSON",
            data: local_data,
            timestamp: Utc::now(),
        })
    }
    
    pub fn delete_user_data(&self) -> Result<(), ComplianceError> {
        // Delete all local telemetry data
        self.clear_local_storage()?;
        
        // Since data is anonymous, cannot delete from analytics service
        // But user can opt-out to prevent future collection
        self.set_telemetry_enabled(false)?;
        Ok(())
    }
}
```

### CCPA Compliance
- **Notice**: Clear disclosure of data collection practices
- **Choice**: Easy opt-out mechanisms
- **Access**: Ability to view collected data
- **Deletion**: Right to delete personal information (N/A for anonymous data)
- **Non-discrimination**: No penalties for opting out

### Privacy Policy Key Points

#### What We Do
- Collect anonymous usage patterns to improve command accuracy
- Aggregate data for statistical analysis and product improvement
- Protect user privacy through design and implementation
- Provide full control over data sharing

#### What We Don't Do
- Collect personal information or identifying data
- Store sensitive commands or file contents
- Share individual user data with third parties
- Use data for advertising or marketing
- Track users across devices or sessions

## Security Measures

### Data Protection
1. **Encryption**: All data encrypted in transit and at rest
2. **Anonymization**: Irreversible anonymization before collection
3. **Access Control**: Strict access controls for analytics systems
4. **Audit Logging**: Complete audit trail of data access
5. **Regular Security Reviews**: Ongoing security assessments

### Incident Response
1. **Detection**: Automated monitoring for security incidents
2. **Response**: Immediate containment and assessment
3. **Notification**: User notification for any data-related incidents
4. **Recovery**: Secure recovery and prevention measures
5. **Documentation**: Complete incident documentation and lessons learned

## Transparency and Trust

### Regular Reporting
1. **Privacy Reports**: Quarterly reports on data practices
2. **Aggregate Statistics**: Public sharing of aggregate insights
3. **Security Updates**: Regular security status updates
4. **Policy Changes**: Clear notification of any policy changes

### Open Source Approach
1. **Code Transparency**: Open source telemetry collection code
2. **Audit Capability**: Public ability to audit privacy practices
3. **Community Input**: Community feedback on privacy policies
4. **External Audits**: Regular third-party privacy audits

### User Education
1. **Clear Documentation**: Easy-to-understand privacy documentation
2. **Regular Reminders**: Periodic reminders about privacy controls
3. **Best Practices**: Education about general privacy best practices
4. **Feedback Channels**: Easy ways to provide privacy feedback

## Implementation Timeline

### Phase 1: Foundation (2 weeks)
- Privacy framework implementation
- Consent management system
- Local data storage with encryption
- Basic anonymization processes

### Phase 2: Collection System (2 weeks)
- Anonymous data collection implementation
- Secure transmission system
- Local privacy controls interface
- GDPR compliance features

### Phase 3: Analytics Service (2 weeks)
- Separate analytics service deployment
- Aggregate data processing
- Statistical analysis capabilities
- Security hardening

### Phase 4: Transparency (1 week)
- Privacy policy finalization
- User documentation
- Transparency reporting setup
- Community feedback integration

## Success Metrics

### Privacy Compliance
- **Zero Personal Data**: No collection of personally identifiable information
- **100% Opt-in**: No data collection without explicit consent
- **Response Time**: <24 hours for privacy requests
- **Compliance Score**: 100% compliance with GDPR/CCPA requirements

### User Trust
- **Opt-in Rate**: Target 30% of users opting in to telemetry
- **Privacy Satisfaction**: >4.5/5 rating for privacy practices
- **Trust Metrics**: High user trust scores in surveys
- **Transparency**: Clear, understandable privacy communications

### Data Quality
- **Anonymization Effectiveness**: 100% of data properly anonymized
- **Data Utility**: Collected data provides actionable insights
- **Processing Accuracy**: Accurate aggregate statistics
- **Security**: Zero security incidents or data breaches

This privacy framework ensures that cmdai can benefit from community insights while maintaining the highest standards of user privacy and regulatory compliance.