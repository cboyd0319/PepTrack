# macOS Keychain Migration Plan

## Current State

PepTrack currently stores its encryption key in a plaintext file:
- **Location:** `~/Library/Application Support/PepTrack/peptrack.key`
- **Format:** 32 random bytes, hex-encoded
- **Security:** File system permissions only

## Goal

Migrate to macOS Keychain for more secure key storage:
- **Benefit:** OS-level encryption and access control
- **Benefit:** No plaintext key file on disk
- **Benefit:** Can require user authentication for key access
- **Benefit:** Integration with macOS security features

## Implementation Plan

### Phase 1: Add Keychain Key Provider

1. Add `security-framework` dependency to `peptrack-core`
2. Implement `KeychainKeyProvider` that implements `KeyProvider` trait
3. Store/retrieve key using `SecItemAdd`/`SecItemCopyMatching`
4. Use service name: `com.peptrack.encryption-key`
5. Use account name: `master-key`

### Phase 2: Migration Utility

1. Create migration function: `migrate_file_key_to_keychain()`
2. On app startup, check if:
   - Old file key exists AND Keychain key doesn't exist → Migrate
   - Both exist → Use Keychain (file is backup)
   - Only Keychain exists → Normal operation
   - Neither exists → Generate new key in Keychain
3. After successful migration, optionally delete old file key

### Phase 3: User Experience

1. Show notification when migration happens
2. Provide option to keep file-based key as backup
3. Add UI preference for re-requiring Touch ID/password for key access

## Code Sketch

```rust
use security_framework::keychain;
use security_framework::item::{ItemClass, ItemSearchOptions};

pub struct KeychainKeyProvider {
    service: String,
    account: String,
}

impl KeychainKeyProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {
            service: "com.peptrack.encryption-key".to_string(),
            account: "master-key".to_string(),
        })
    }

    fn load_or_create_key(&self) -> Result<Vec<u8>> {
        // Try to load existing key
        if let Ok(key) = self.load_from_keychain() {
            return Ok(key);
        }

        // Generate and store new key
        let key = self.generate_key()?;
        self.store_in_keychain(&key)?;
        Ok(key)
    }

    fn load_from_keychain(&self) -> Result<Vec<u8>> {
        // Use SecItemCopyMatching to retrieve
        todo!()
    }

    fn store_in_keychain(&self, key: &[u8]) -> Result<()> {
        // Use SecItemAdd to store
        // Set kSecAttrAccessible to kSecAttrAccessibleAfterFirstUnlock
        todo!()
    }
}

impl KeyProvider for KeychainKeyProvider {
    fn key_material(&self) -> Result<KeyMaterial> {
        let bytes = self.load_or_create_key()?;
        KeyMaterial::new(bytes)
    }
}
```

## Testing Strategy

1. Unit tests with mock keychain (if possible)
2. Integration test: Store → Retrieve → Verify
3. Migration test: File key → Keychain → Verify same key
4. Manual testing on fresh macOS install

## Rollout

1. Ship as opt-in feature first
2. Add toggle in settings: "Use macOS Keychain for encryption keys"
3. After validation period, make it default for new installations
4. Provide clear migration path for existing users

## References

- [security-framework crate](https://docs.rs/security-framework/)
- [macOS Keychain Services](https://developer.apple.com/documentation/security/keychain_services)
- [SecItemAdd documentation](https://developer.apple.com/documentation/security/1401659-secitemadd)

## Estimated Effort

- **Implementation:** 4-6 hours
- **Testing:** 2-3 hours
- **Documentation:** 1-2 hours
- **Total:** ~1 day of focused work
