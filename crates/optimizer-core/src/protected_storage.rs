//! Protected local storage contracts for rollback backups and local tokens.

use std::{collections::BTreeMap, fmt};

const MAX_RECORD_KEY_LEN: usize = 128;

/// Sensitive local data categories that must be stored through OS protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensitiveDataPurpose {
    /// Rollback or backup data needed to restore user/system state.
    RollbackBackup,
    /// Locally cached token or credential-like material.
    LocalToken,
}

impl SensitiveDataPurpose {
    /// Returns a stable purpose label for storage metadata and audit records.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RollbackBackup => "rollback_backup",
            Self::LocalToken => "local_token",
        }
    }
}

/// Validated key for an item in protected local storage.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtectedRecordKey {
    value: String,
}

impl ProtectedRecordKey {
    /// Creates a storage key from a scoped identifier, not a filesystem path.
    pub fn new(value: impl Into<String>) -> Result<Self, ProtectedStorageError> {
        let value = value.into();
        let trimmed = value.trim();

        if trimmed.is_empty()
            || trimmed.len() > MAX_RECORD_KEY_LEN
            || !trimmed.bytes().all(is_allowed_record_key_byte)
        {
            return Err(ProtectedStorageError::invalid_key());
        }

        Ok(Self {
            value: trimmed.to_owned(),
        })
    }

    /// Returns the validated key string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

fn is_allowed_record_key_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
}

/// Opaque protected bytes and metadata stored by a local persistence backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedBlob {
    purpose: SensitiveDataPurpose,
    protector_backend: &'static str,
    protected_bytes: Vec<u8>,
}

impl ProtectedBlob {
    fn new(
        purpose: SensitiveDataPurpose,
        protector_backend: &'static str,
        protected_bytes: Vec<u8>,
    ) -> Result<Self, ProtectedStorageError> {
        if protected_bytes.is_empty() {
            return Err(ProtectedStorageError::protection_failed());
        }

        Ok(Self {
            purpose,
            protector_backend,
            protected_bytes,
        })
    }

    /// Returns the sensitive-data purpose attached to this protected blob.
    #[must_use]
    pub const fn purpose(&self) -> SensitiveDataPurpose {
        self.purpose
    }

    /// Returns the backend that produced the protected bytes.
    #[must_use]
    pub const fn protector_backend(&self) -> &'static str {
        self.protector_backend
    }

    /// Returns the encrypted or OS-protected payload bytes.
    #[must_use]
    pub fn protected_bytes(&self) -> &[u8] {
        &self.protected_bytes
    }
}

/// Platform protection primitive used before sensitive data reaches storage.
pub trait LocalDataProtector {
    /// Returns a stable backend label, such as `windows-dpapi`.
    fn backend(&self) -> &'static str;

    /// Protects plaintext bytes for local storage.
    fn protect(
        &self,
        purpose: SensitiveDataPurpose,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProtectedStorageError>;

    /// Restores plaintext bytes from a protected local payload.
    fn unprotect(
        &self,
        purpose: SensitiveDataPurpose,
        protected_bytes: &[u8],
    ) -> Result<Vec<u8>, ProtectedStorageError>;
}

/// Minimal adapter contract for protected local persistence.
pub trait ProtectedStorageAdapter {
    /// Stores sensitive bytes only after passing through the configured protector.
    fn put(
        &mut self,
        key: ProtectedRecordKey,
        purpose: SensitiveDataPurpose,
        plaintext: &[u8],
    ) -> Result<(), ProtectedStorageError>;

    /// Loads and unprotects sensitive bytes for the expected purpose.
    fn get(
        &self,
        key: &ProtectedRecordKey,
        purpose: SensitiveDataPurpose,
    ) -> Result<Vec<u8>, ProtectedStorageError>;

    /// Removes a protected record if present.
    fn remove(&mut self, key: &ProtectedRecordKey) -> Option<ProtectedBlob>;
}

/// In-memory protected storage used by tests and future persistence adapters.
#[derive(Debug, Clone)]
pub struct InMemoryProtectedStorage<P> {
    protector: P,
    records: BTreeMap<ProtectedRecordKey, ProtectedBlob>,
}

impl<P> InMemoryProtectedStorage<P>
where
    P: LocalDataProtector,
{
    /// Creates an empty protected store backed by a platform protector.
    #[must_use]
    pub fn new(protector: P) -> Self {
        Self {
            protector,
            records: BTreeMap::new(),
        }
    }

    /// Returns the protected blob metadata for a key, without unprotecting it.
    #[must_use]
    pub fn protected_blob(&self, key: &ProtectedRecordKey) -> Option<&ProtectedBlob> {
        self.records.get(key)
    }

    /// Returns the number of protected records in this adapter.
    #[must_use]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns whether this adapter currently has no protected records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl<P> ProtectedStorageAdapter for InMemoryProtectedStorage<P>
where
    P: LocalDataProtector,
{
    fn put(
        &mut self,
        key: ProtectedRecordKey,
        purpose: SensitiveDataPurpose,
        plaintext: &[u8],
    ) -> Result<(), ProtectedStorageError> {
        if plaintext.is_empty() {
            return Err(ProtectedStorageError::empty_plaintext());
        }

        let protected_bytes = self.protector.protect(purpose, plaintext)?;
        let blob = ProtectedBlob::new(purpose, self.protector.backend(), protected_bytes)?;
        self.records.insert(key, blob);

        Ok(())
    }

    fn get(
        &self,
        key: &ProtectedRecordKey,
        purpose: SensitiveDataPurpose,
    ) -> Result<Vec<u8>, ProtectedStorageError> {
        let blob = self
            .records
            .get(key)
            .ok_or_else(ProtectedStorageError::missing_record)?;

        if blob.purpose() != purpose {
            return Err(ProtectedStorageError::purpose_mismatch());
        }

        self.protector.unprotect(purpose, blob.protected_bytes())
    }

    fn remove(&mut self, key: &ProtectedRecordKey) -> Option<ProtectedBlob> {
        self.records.remove(key)
    }
}

/// Error returned by protected storage validation and adapter operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectedStorageErrorReason {
    /// Storage key is empty, too long, or contains unsafe characters.
    InvalidKey,
    /// Sensitive plaintext was empty.
    EmptyPlaintext,
    /// Protector returned no protected payload.
    ProtectionFailed,
    /// Protector could not restore the payload.
    UnprotectFailed,
    /// Requested protected record does not exist.
    MissingRecord,
    /// Caller requested a record under the wrong sensitive-data purpose.
    PurposeMismatch,
}

impl ProtectedStorageErrorReason {
    /// Returns a stable reason string for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidKey => "invalid_key",
            Self::EmptyPlaintext => "empty_plaintext",
            Self::ProtectionFailed => "protection_failed",
            Self::UnprotectFailed => "unprotect_failed",
            Self::MissingRecord => "missing_record",
            Self::PurposeMismatch => "purpose_mismatch",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::InvalidKey => "Protected storage key failed validation",
            Self::EmptyPlaintext => "Protected storage plaintext cannot be empty",
            Self::ProtectionFailed => "Local data protection failed",
            Self::UnprotectFailed => "Local data unprotect failed",
            Self::MissingRecord => "Protected storage record was not found",
            Self::PurposeMismatch => "Protected storage purpose did not match",
        }
    }
}

/// Structured error returned by protected storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectedStorageError {
    reason: ProtectedStorageErrorReason,
}

impl ProtectedStorageError {
    const fn new(reason: ProtectedStorageErrorReason) -> Self {
        Self { reason }
    }

    fn invalid_key() -> Self {
        Self::new(ProtectedStorageErrorReason::InvalidKey)
    }

    fn empty_plaintext() -> Self {
        Self::new(ProtectedStorageErrorReason::EmptyPlaintext)
    }

    /// Creates a protection failure error for platform protector implementations.
    #[must_use]
    pub const fn protection_failed() -> Self {
        Self::new(ProtectedStorageErrorReason::ProtectionFailed)
    }

    /// Creates an unprotect failure error for platform protector implementations.
    #[must_use]
    pub const fn unprotect_failed() -> Self {
        Self::new(ProtectedStorageErrorReason::UnprotectFailed)
    }

    fn missing_record() -> Self {
        Self::new(ProtectedStorageErrorReason::MissingRecord)
    }

    fn purpose_mismatch() -> Self {
        Self::new(ProtectedStorageErrorReason::PurposeMismatch)
    }

    /// Returns the protected storage error reason.
    #[must_use]
    pub const fn reason(self) -> ProtectedStorageErrorReason {
        self.reason
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        self.reason.message()
    }
}

impl fmt::Display for ProtectedStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.message())
    }
}

impl std::error::Error for ProtectedStorageError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    struct TestProtector;

    impl LocalDataProtector for TestProtector {
        fn backend(&self) -> &'static str {
            "test-protector"
        }

        fn protect(
            &self,
            purpose: SensitiveDataPurpose,
            plaintext: &[u8],
        ) -> Result<Vec<u8>, ProtectedStorageError> {
            let mut protected = Vec::from(purpose.as_str().as_bytes());
            protected.push(b':');
            protected.extend(plaintext.iter().rev());
            Ok(protected)
        }

        fn unprotect(
            &self,
            purpose: SensitiveDataPurpose,
            protected_bytes: &[u8],
        ) -> Result<Vec<u8>, ProtectedStorageError> {
            let prefix = format!("{}:", purpose.as_str());

            let payload = protected_bytes
                .strip_prefix(prefix.as_bytes())
                .ok_or_else(ProtectedStorageError::unprotect_failed)?;

            Ok(payload.iter().rev().copied().collect())
        }
    }

    #[test]
    fn protects_backup_bytes_before_storage_and_restores_them() {
        let key = ProtectedRecordKey::new("rollback:snapshot-001")
            .expect("key should be valid");
        let plaintext = b"{\"registry\":\"backup\"}";
        let mut store = InMemoryProtectedStorage::new(TestProtector);

        store
            .put(key.clone(), SensitiveDataPurpose::RollbackBackup, plaintext)
            .expect("backup should be protected before storage");

        let stored = store
            .protected_blob(&key)
            .expect("protected blob should be stored");

        assert_eq!(stored.purpose(), SensitiveDataPurpose::RollbackBackup);
        assert_eq!(stored.protector_backend(), "test-protector");
        assert_ne!(stored.protected_bytes(), plaintext);

        let restored = store
            .get(&key, SensitiveDataPurpose::RollbackBackup)
            .expect("backup should unprotect");

        assert_eq!(restored, plaintext);
    }

    #[test]
    fn protects_local_token_bytes_under_token_purpose() {
        let key = ProtectedRecordKey::new("token:benchmark-sync")
            .expect("key should be valid");
        let plaintext = b"refresh-token";
        let mut store = InMemoryProtectedStorage::new(TestProtector);

        store
            .put(key.clone(), SensitiveDataPurpose::LocalToken, plaintext)
            .expect("token should be protected before storage");

        let stored = store
            .protected_blob(&key)
            .expect("protected token should be stored");

        assert_eq!(stored.purpose(), SensitiveDataPurpose::LocalToken);
        assert!(!stored.protected_bytes().windows(plaintext.len()).any(|window| {
            window == plaintext
        }));
        assert_eq!(
            store
                .get(&key, SensitiveDataPurpose::LocalToken)
                .expect("token should unprotect"),
            plaintext
        );
    }

    #[test]
    fn denies_empty_plaintext_and_unsafe_keys() {
        let mut store = InMemoryProtectedStorage::new(TestProtector);
        let key = ProtectedRecordKey::new("rollback:snapshot-001")
            .expect("key should be valid");

        let error = store
            .put(key, SensitiveDataPurpose::RollbackBackup, b"")
            .expect_err("empty sensitive data should not be stored");

        assert_eq!(error.reason(), ProtectedStorageErrorReason::EmptyPlaintext);
        assert_eq!(store.len(), 0);

        let error = ProtectedRecordKey::new("..\\secrets")
            .expect_err("filesystem-like key should be rejected");

        assert_eq!(error.reason(), ProtectedStorageErrorReason::InvalidKey);
    }

    #[test]
    fn denies_reading_record_with_wrong_sensitive_purpose() {
        let key = ProtectedRecordKey::new("token:api")
            .expect("key should be valid");
        let mut store = InMemoryProtectedStorage::new(TestProtector);

        store
            .put(key.clone(), SensitiveDataPurpose::LocalToken, b"token")
            .expect("token should be protected before storage");

        let error = store
            .get(&key, SensitiveDataPurpose::RollbackBackup)
            .expect_err("purpose mismatch should be denied");

        assert_eq!(error.reason(), ProtectedStorageErrorReason::PurposeMismatch);
    }
}
