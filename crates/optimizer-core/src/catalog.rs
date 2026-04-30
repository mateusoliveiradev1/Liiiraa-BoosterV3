//! Validated tweak catalog loading and registry lookup.

use std::{collections::BTreeMap, fmt};

use crate::tweak_contracts::{
    SourceLink, TweakDefinition, TweakId, TweakOperation, TweakStep,
};

/// The only catalog schema version supported by this build.
pub const SUPPORTED_CATALOG_SCHEMA_VERSION: &str = "1";

/// Complete catalog payload after transport parsing but before validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakCatalog {
    /// Schema version carried by the catalog payload.
    pub schema_version: String,
    /// Source and trust metadata attached to the payload.
    pub source: CatalogSource,
    /// Tweak definitions supplied by this catalog.
    pub definitions: Vec<TweakDefinition>,
}

impl TweakCatalog {
    /// Creates a catalog payload for validation.
    #[must_use]
    pub fn new(
        schema_version: impl Into<String>,
        source: CatalogSource,
        definitions: Vec<TweakDefinition>,
    ) -> Self {
        Self {
            schema_version: schema_version.into(),
            source,
            definitions,
        }
    }
}

/// Trust metadata for a tweak catalog source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogSource {
    id: String,
    kind: CatalogSourceKind,
    revoked: bool,
}

impl CatalogSource {
    /// Creates a built-in catalog source packaged with the signed app.
    #[must_use]
    pub fn embedded(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: CatalogSourceKind::Embedded,
            revoked: false,
        }
    }

    /// Creates a remote source that already passed signature and integrity checks.
    #[must_use]
    pub fn signed_remote(
        id: impl Into<String>,
        signature_ref: impl Into<String>,
        integrity_ref: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: CatalogSourceKind::SignedRemote {
                signature_ref: signature_ref.into(),
                integrity_ref: integrity_ref.into(),
            },
            revoked: false,
        }
    }

    /// Creates a remote source without cryptographic trust metadata.
    #[must_use]
    pub fn unsigned_remote(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: CatalogSourceKind::UnsignedRemote,
            revoked: false,
        }
    }

    /// Marks this source as revoked by a release or catalog rollback.
    #[must_use]
    pub fn revoked(mut self) -> Self {
        self.revoked = true;
        self
    }

    /// Returns the stable source identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the source trust kind.
    #[must_use]
    pub fn kind(&self) -> &CatalogSourceKind {
        &self.kind
    }

    /// Returns whether this catalog source was marked revoked.
    #[must_use]
    pub const fn is_revoked(&self) -> bool {
        self.revoked
    }
}

/// Trust class for a catalog source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogSourceKind {
    /// Catalog is packaged with the signed application.
    Embedded,
    /// Remote catalog passed signature and integrity verification before loading.
    SignedRemote {
        /// Signature, key, or release reference used by the transport layer.
        signature_ref: String,
        /// Integrity reference, currently expected to be a `sha256:` digest.
        integrity_ref: String,
    },
    /// Remote catalog without proof of signature or integrity.
    UnsignedRemote,
}

/// Registry produced by a validated catalog load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakRegistry {
    schema_version: String,
    source: CatalogSource,
    definitions: BTreeMap<TweakId, TweakDefinition>,
}

impl TweakRegistry {
    /// Returns the catalog schema version used by this registry.
    #[must_use]
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Returns the validated source metadata for this registry.
    #[must_use]
    pub fn source(&self) -> &CatalogSource {
        &self.source
    }

    /// Returns the number of tweak definitions in this registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns whether this registry has no tweak definitions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Returns true when the registry contains a tweak ID.
    #[must_use]
    pub fn contains(&self, tweak_id: &str) -> bool {
        self.definitions.contains_key(tweak_id)
    }

    /// Looks up one tweak definition by ID.
    #[must_use]
    pub fn get(&self, tweak_id: &str) -> Option<&TweakDefinition> {
        self.definitions.get(tweak_id)
    }

    /// Iterates over validated tweak definitions in stable ID order.
    pub fn iter(&self) -> impl Iterator<Item = &TweakDefinition> {
        self.definitions.values()
    }

    /// Iterates over definitions eligible for the default optimization flow.
    pub fn default_candidates(&self) -> impl Iterator<Item = &TweakDefinition> {
        self.iter()
            .filter(|definition| definition.is_default_candidate())
    }
}

/// Loads and validates a tweak catalog into a stable registry.
pub fn load_tweak_catalog(catalog: TweakCatalog) -> Result<TweakRegistry, CatalogValidationError> {
    validate_schema_version(&catalog.schema_version)?;
    validate_catalog_source(&catalog.source)?;

    if catalog.definitions.is_empty() {
        return Err(CatalogValidationError::new(
            CatalogValidationErrorReason::EmptyCatalog,
        ));
    }

    let mut definitions = BTreeMap::new();

    for definition in catalog.definitions {
        validate_definition(&definition)?;

        if definitions.contains_key(&definition.id) {
            return Err(CatalogValidationError::with_tweak(
                CatalogValidationErrorReason::DuplicateTweakId,
                definition.id.clone(),
            ));
        }

        definitions.insert(definition.id.clone(), definition);
    }

    Ok(TweakRegistry {
        schema_version: catalog.schema_version,
        source: catalog.source,
        definitions,
    })
}

fn validate_schema_version(schema_version: &str) -> Result<(), CatalogValidationError> {
    if schema_version == SUPPORTED_CATALOG_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(CatalogValidationError::with_detail(
            CatalogValidationErrorReason::UnsupportedSchemaVersion,
            schema_version,
        ))
    }
}

fn validate_catalog_source(source: &CatalogSource) -> Result<(), CatalogValidationError> {
    if source.revoked {
        return Err(CatalogValidationError::with_detail(
            CatalogValidationErrorReason::RevokedSource,
            source.id(),
        ));
    }

    if !is_valid_identifier(source.id()) {
        return Err(CatalogValidationError::with_detail(
            CatalogValidationErrorReason::InvalidSource,
            source.id(),
        ));
    }

    match source.kind() {
        CatalogSourceKind::Embedded => Ok(()),
        CatalogSourceKind::SignedRemote {
            signature_ref,
            integrity_ref,
        } => {
            if is_valid_identifier(signature_ref) && is_valid_sha256_ref(integrity_ref) {
                Ok(())
            } else {
                Err(CatalogValidationError::with_detail(
                    CatalogValidationErrorReason::InvalidSource,
                    source.id(),
                ))
            }
        }
        CatalogSourceKind::UnsignedRemote => Err(CatalogValidationError::with_detail(
            CatalogValidationErrorReason::UntrustedSource,
            source.id(),
        )),
    }
}

fn validate_definition(definition: &TweakDefinition) -> Result<(), CatalogValidationError> {
    if !is_valid_tweak_id(&definition.id) {
        return Err(CatalogValidationError::with_tweak(
            CatalogValidationErrorReason::InvalidTweakId,
            definition.id.clone(),
        ));
    }

    if definition.source_links.is_empty() {
        return Err(CatalogValidationError::with_tweak(
            CatalogValidationErrorReason::MissingSourceLinks,
            definition.id.clone(),
        ));
    }

    for source_link in &definition.source_links {
        validate_source_link(source_link).map_err(|error| error.with_tweak_id(&definition.id))?;
    }

    for step in definition_steps(definition) {
        validate_step_operations(&definition.id, step)?;
    }

    if definition.is_blocked_guardrail() && definition.default_enabled {
        return Err(CatalogValidationError::with_tweak(
            CatalogValidationErrorReason::UnsafeDefaultGuardrail,
            definition.id.clone(),
        ));
    }

    Ok(())
}

fn validate_source_link(source_link: &SourceLink) -> Result<(), CatalogValidationError> {
    let title = source_link.title.trim();
    let url = source_link.url.trim();

    if title.is_empty() || !(url.starts_with("https://") || url.starts_with("local:")) {
        Err(CatalogValidationError::with_detail(
            CatalogValidationErrorReason::InvalidSourceLink,
            source_link.url.as_str(),
        ))
    } else {
        Ok(())
    }
}

fn definition_steps(definition: &TweakDefinition) -> [&TweakStep; 7] {
    [
        &definition.detect,
        &definition.precheck,
        &definition.plan,
        &definition.backup,
        &definition.apply,
        &definition.verify,
        &definition.rollback,
    ]
}

fn validate_step_operations(
    tweak_id: &str,
    step: &TweakStep,
) -> Result<(), CatalogValidationError> {
    for operation in &step.operations {
        validate_operation(tweak_id, operation)?;
    }

    Ok(())
}

fn validate_operation(
    tweak_id: &str,
    operation: &TweakOperation,
) -> Result<(), CatalogValidationError> {
    if looks_like_arbitrary_script(&operation.target)
        || operation
            .value
            .as_deref()
            .is_some_and(looks_like_arbitrary_script)
    {
        return Err(CatalogValidationError::with_tweak(
            CatalogValidationErrorReason::ArbitraryScriptOperation,
            tweak_id.to_owned(),
        ));
    }

    Ok(())
}

fn is_valid_tweak_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
}

fn is_valid_identifier(value: &str) -> bool {
    let value = value.trim();

    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':'))
}

fn is_valid_sha256_ref(value: &str) -> bool {
    let Some(hash) = value.strip_prefix("sha256:") else {
        return false;
    };

    hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn looks_like_arbitrary_script(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();

    value.starts_with("script:")
        || value.starts_with("shell:")
        || value.starts_with("exec:")
        || value.contains("powershell")
        || value.contains("cmd.exe")
        || value.contains("bash -")
        || value.contains(".ps1")
        || (value.contains("curl ") && value.contains('|'))
}

/// Reason a catalog failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogValidationErrorReason {
    /// Catalog schema version is not supported by this build.
    UnsupportedSchemaVersion,
    /// Catalog source metadata is malformed.
    InvalidSource,
    /// Catalog source was marked revoked.
    RevokedSource,
    /// Catalog came from a source without signature or integrity trust.
    UntrustedSource,
    /// Catalog contained no tweak definitions.
    EmptyCatalog,
    /// A tweak ID appeared more than once.
    DuplicateTweakId,
    /// A tweak ID was empty or contained unsafe characters.
    InvalidTweakId,
    /// A definition did not include source links.
    MissingSourceLinks,
    /// A source link was missing or did not use an accepted scheme.
    InvalidSourceLink,
    /// A catalog operation tried to embed shell or script execution.
    ArbitraryScriptOperation,
    /// A blocked guardrail was enabled as a default tweak.
    UnsafeDefaultGuardrail,
}

impl CatalogValidationErrorReason {
    /// Returns a stable reason string for logs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion => "unsupported_schema_version",
            Self::InvalidSource => "invalid_source",
            Self::RevokedSource => "revoked_source",
            Self::UntrustedSource => "untrusted_source",
            Self::EmptyCatalog => "empty_catalog",
            Self::DuplicateTweakId => "duplicate_tweak_id",
            Self::InvalidTweakId => "invalid_tweak_id",
            Self::MissingSourceLinks => "missing_source_links",
            Self::InvalidSourceLink => "invalid_source_link",
            Self::ArbitraryScriptOperation => "arbitrary_script_operation",
            Self::UnsafeDefaultGuardrail => "unsafe_default_guardrail",
        }
    }

    /// Returns a short human-readable message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::UnsupportedSchemaVersion => "Catalog schema version is not supported",
            Self::InvalidSource => "Catalog source metadata failed validation",
            Self::RevokedSource => "Catalog source was revoked",
            Self::UntrustedSource => "Catalog source is not trusted",
            Self::EmptyCatalog => "Catalog must contain at least one tweak definition",
            Self::DuplicateTweakId => "Catalog contains a duplicate tweak ID",
            Self::InvalidTweakId => "Catalog contains an invalid tweak ID",
            Self::MissingSourceLinks => "Tweak definition is missing source links",
            Self::InvalidSourceLink => "Tweak source link failed validation",
            Self::ArbitraryScriptOperation => "Tweak operation contains script-like content",
            Self::UnsafeDefaultGuardrail => "Blocked guardrail cannot be enabled by default",
        }
    }
}

/// Structured catalog validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogValidationError {
    reason: CatalogValidationErrorReason,
    tweak_id: Option<TweakId>,
    detail: Option<String>,
}

impl CatalogValidationError {
    const fn new(reason: CatalogValidationErrorReason) -> Self {
        Self {
            reason,
            tweak_id: None,
            detail: None,
        }
    }

    fn with_tweak(reason: CatalogValidationErrorReason, tweak_id: TweakId) -> Self {
        Self {
            reason,
            tweak_id: Some(tweak_id),
            detail: None,
        }
    }

    fn with_detail(reason: CatalogValidationErrorReason, detail: impl Into<String>) -> Self {
        Self {
            reason,
            tweak_id: None,
            detail: Some(detail.into()),
        }
    }

    fn with_tweak_id(mut self, tweak_id: &str) -> Self {
        self.tweak_id = Some(tweak_id.to_owned());
        self
    }

    /// Returns the validation failure reason.
    #[must_use]
    pub const fn reason(&self) -> CatalogValidationErrorReason {
        self.reason
    }

    /// Returns the tweak ID associated with the failure, when known.
    #[must_use]
    pub fn tweak_id(&self) -> Option<&str> {
        self.tweak_id.as_deref()
    }

    /// Returns extra source or schema detail associated with the failure.
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl fmt::Display for CatalogValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.reason.message())?;

        if let Some(tweak_id) = self.tweak_id() {
            write!(formatter, " ({tweak_id})")?;
        }

        if let Some(detail) = self.detail() {
            write!(formatter, " [{detail}]")?;
        }

        Ok(())
    }
}

impl std::error::Error for CatalogValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tweak_contracts::{
        CompatibilityRule, CompatibilityTarget, EvidenceLevel, ExpectedImpact, ImpactDirection,
        LaptopPolicy, MeasurementMetric, MeasurementPlan, PowerSourcePolicy, RebootPolicy,
        RollbackKind, SessionScope, SourceLink, TweakCategory, TweakMode, TweakOperationKind,
        TweakDefinition, TweakOperation, TweakRisk, TweakStep, TweakStepKind, TweakTestCase,
        TweakTestPlan,
    };

    fn catalog_with(definitions: Vec<TweakDefinition>) -> TweakCatalog {
        TweakCatalog::new(
            SUPPORTED_CATALOG_SCHEMA_VERSION,
            CatalogSource::embedded("catalog:embedded:v1"),
            definitions,
        )
    }

    fn valid_sha256_ref() -> String {
        format!("sha256:{}", "a".repeat(64))
    }

    fn measurement_plan() -> MeasurementPlan {
        MeasurementPlan {
            benchmark_required: false,
            metrics: vec![MeasurementMetric {
                key: "scan.complete".to_owned(),
                label: "Scan completed".to_owned(),
                unit: "bool".to_owned(),
            }],
            notes: vec!["Read-only catalog fixture.".to_owned()],
        }
    }

    fn expected_impact() -> ExpectedImpact {
        ExpectedImpact {
            metric: "system visibility".to_owned(),
            direction: ImpactDirection::Informational,
            evidence: EvidenceLevel::Official,
            summary: "Makes planning decisions use current system state.".to_owned(),
        }
    }

    fn test_plan() -> TweakTestPlan {
        TweakTestPlan {
            cases: vec![TweakTestCase {
                id: "catalog-loads".to_owned(),
                covers: "catalog schema validation".to_owned(),
            }],
            fixtures: vec!["catalog:minimal-v1".to_owned()],
            requires_live_windows: false,
        }
    }

    fn read_only_step(kind: TweakStepKind, summary: &str) -> TweakStep {
        TweakStep::read_only(kind, summary)
    }

    fn source_links() -> Vec<SourceLink> {
        vec![SourceLink {
            title: "V1 tweak matrix".to_owned(),
            url: "local:v1-tweak-matrix".to_owned(),
            evidence: EvidenceLevel::Official,
        }]
    }

    fn compatibility_rule() -> CompatibilityRule {
        CompatibilityRule {
            target: CompatibilityTarget::Windows,
            expression: "windows-10+".to_owned(),
            reason: "Windows is the target platform.".to_owned(),
        }
    }

    fn inventory_definition() -> TweakDefinition {
        TweakDefinition {
            id: "sys.scan.inventory".to_owned(),
            title: "System inventory scan".to_owned(),
            summary: "Reads system state before planning tweaks.".to_owned(),
            category: TweakCategory::BaselineHealth,
            mode: TweakMode::Safe,
            risk: TweakRisk::Low,
            default_enabled: true,
            session_scope: SessionScope::RecommendationOnly,
            rollback_kind: RollbackKind::NotNeededReadonly,
            requires_admin: false,
            reboot: RebootPolicy::None,
            supported_os: vec!["windows-10+".to_owned()],
            supported_hardware: vec![compatibility_rule()],
            supported_drivers: Vec::new(),
            unsupported_when: Vec::new(),
            conflicts_with: Vec::new(),
            laptop_policy: LaptopPolicy::SameAsDesktop,
            power_source_policy: PowerSourcePolicy::Any,
            source_links: source_links(),
            evidence_level: EvidenceLevel::Official,
            measurement_plan: measurement_plan(),
            expected_impact: expected_impact(),
            known_side_effects: Vec::new(),
            anti_cheat_notes: vec!["Read-only; no game or anti-cheat mutation.".to_owned()],
            game_closed_required: false,
            user_disclosure: "Inventory is read-only.".to_owned(),
            r#do: vec!["Read current system state.".to_owned()],
            dont: vec!["Do not mutate state during scan.".to_owned()],
            detect: read_only_step(TweakStepKind::Detect, "Read system inventory."),
            precheck: read_only_step(TweakStepKind::Precheck, "Check API availability."),
            plan: read_only_step(TweakStepKind::Plan, "Build a read-only finding."),
            backup: read_only_step(TweakStepKind::Backup, "No backup required."),
            apply: read_only_step(TweakStepKind::Apply, "No apply operation."),
            verify: read_only_step(TweakStepKind::Verify, "Confirm scan completed."),
            rollback: read_only_step(TweakStepKind::Rollback, "No rollback required."),
            tests: test_plan(),
        }
    }

    #[test]
    fn loads_valid_embedded_catalog_into_registry() {
        let registry = load_tweak_catalog(catalog_with(vec![inventory_definition()]))
            .expect("valid catalog should load");

        assert_eq!(registry.schema_version(), SUPPORTED_CATALOG_SCHEMA_VERSION);
        assert_eq!(registry.source().id(), "catalog:embedded:v1");
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("sys.scan.inventory"));
        assert_eq!(
            registry
                .default_candidates()
                .map(|definition| definition.id.as_str())
                .collect::<Vec<_>>(),
            vec!["sys.scan.inventory"]
        );
    }

    #[test]
    fn accepts_signed_remote_catalog_source() {
        let catalog = TweakCatalog::new(
            SUPPORTED_CATALOG_SCHEMA_VERSION,
            CatalogSource::signed_remote("catalog:stable:v1", "sig:stable:001", valid_sha256_ref()),
            vec![inventory_definition()],
        );

        let registry = load_tweak_catalog(catalog).expect("signed source should load");

        assert_eq!(registry.source().id(), "catalog:stable:v1");
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let catalog = TweakCatalog::new(
            "2",
            CatalogSource::embedded("catalog:embedded:v2"),
            vec![inventory_definition()],
        );

        let error = load_tweak_catalog(catalog).expect_err("schema must be supported");

        assert_eq!(
            error.reason(),
            CatalogValidationErrorReason::UnsupportedSchemaVersion
        );
        assert_eq!(error.detail(), Some("2"));
    }

    #[test]
    fn rejects_unsigned_and_revoked_sources() {
        let unsigned = TweakCatalog::new(
            SUPPORTED_CATALOG_SCHEMA_VERSION,
            CatalogSource::unsigned_remote("catalog:stable:v1"),
            vec![inventory_definition()],
        );

        let unsigned_error =
            load_tweak_catalog(unsigned).expect_err("unsigned remote source must be denied");

        assert_eq!(
            unsigned_error.reason(),
            CatalogValidationErrorReason::UntrustedSource
        );

        let revoked = TweakCatalog::new(
            SUPPORTED_CATALOG_SCHEMA_VERSION,
            CatalogSource::embedded("catalog:embedded:v1").revoked(),
            vec![inventory_definition()],
        );

        let revoked_error =
            load_tweak_catalog(revoked).expect_err("revoked catalog must be denied");

        assert_eq!(
            revoked_error.reason(),
            CatalogValidationErrorReason::RevokedSource
        );
    }

    #[test]
    fn rejects_duplicate_tweak_ids() {
        let error = load_tweak_catalog(catalog_with(vec![
            inventory_definition(),
            inventory_definition(),
        ]))
        .expect_err("duplicate ids must be denied");

        assert_eq!(
            error.reason(),
            CatalogValidationErrorReason::DuplicateTweakId
        );
    }

    #[test]
    fn rejects_definitions_without_source_links() {
        let mut definition = inventory_definition();
        definition.source_links.clear();

        let error = load_tweak_catalog(catalog_with(vec![definition]))
            .expect_err("source links are required");

        assert_eq!(
            error.reason(),
            CatalogValidationErrorReason::MissingSourceLinks
        );
        assert_eq!(error.tweak_id(), Some("sys.scan.inventory"));
    }

    #[test]
    fn rejects_script_like_catalog_operations() {
        let mut definition = inventory_definition();
        definition.apply.mutates_system = true;
        definition.apply.operations.push(TweakOperation {
            kind: TweakOperationKind::Write,
            target: "shell:powershell".to_owned(),
            value: Some("powershell -NoProfile -EncodedCommand AAA".to_owned()),
        });

        let error = load_tweak_catalog(catalog_with(vec![definition]))
            .expect_err("catalogs must not carry shell scripts");

        assert_eq!(
            error.reason(),
            CatalogValidationErrorReason::ArbitraryScriptOperation
        );
        assert_eq!(error.tweak_id(), Some("sys.scan.inventory"));
    }
}
