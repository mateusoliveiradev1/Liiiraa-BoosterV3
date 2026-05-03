use optimizer_core::ipc::{
    ipc_allowlist, validate_ipc_request, IpcCommandPayload, IpcDenial, IpcRequester, RawIpcRequest,
    SecurityStatusPayload,
};
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::process::Command;
use windows_api::{scan_system, SystemScanError, SystemScanReport};

const LIVE_RESOURCE_SNAPSHOT_COMMAND_ID: &str = "system.resources.live";
const SYSTEM_SCAN_COMMAND_ID: &str = "system.scan.read_only";

#[cfg(windows)]
const LIVE_RESOURCE_SNAPSHOT_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Invoke-LiveSection([scriptblock]$body, $fallback) {
    try {
        & $body
    } catch {
        $fallback
    }
}

function Convert-NullableUInt32($value) {
    if ($null -eq $value) {
        $null
    } else {
        [UInt32]$value
    }
}

function Convert-NullableUInt64($value) {
    if ($null -eq $value) {
        $null
    } else {
        [UInt64]$value
    }
}

function Convert-NullableDouble($value) {
    if ($null -eq $value) {
        $null
    } else {
        [Math]::Round([double]$value, 1)
    }
}

function Convert-NullableString($value) {
    if ($null -eq $value) {
        $null
    } else {
        [string]$value
    }
}

function Get-UsedPercent($total, $free) {
    if ($null -eq $total -or $null -eq $free -or [double]$total -le 0) {
        $null
    } else {
        [Math]::Round((1 - ([double]$free / [double]$total)) * 100, 1)
    }
}

function Get-ProcessCpuSample($logicalProcessorCount, $sampleMilliseconds) {
    if ($null -eq $logicalProcessorCount -or [double]$logicalProcessorCount -le 0) {
        return $null
    }

    $before = @{}

    foreach ($process in @(Get-Process -ErrorAction Stop)) {
        if ($null -ne $process.CPU) {
            $before[[int]$process.Id] = [double]$process.CPU
        }
    }

    Start-Sleep -Milliseconds $sampleMilliseconds

    $deltaSeconds = 0.0

    foreach ($process in @(Get-Process -ErrorAction Stop)) {
        if ($null -ne $process.CPU -and $before.ContainsKey([int]$process.Id)) {
            $delta = [double]$process.CPU - $before[[int]$process.Id]

            if ($delta -gt 0) {
                $deltaSeconds += $delta
            }
        }
    }

    $usage = ($deltaSeconds / ([double]$sampleMilliseconds / 1000.0) / [double]$logicalProcessorCount) * 100.0
    [Math]::Round([Math]::Min(100.0, [Math]::Max(0.0, $usage)), 1)
}

$cpuRegistry = Invoke-LiveSection {
    Get-ItemProperty -LiteralPath 'HKLM:\HARDWARE\DESCRIPTION\System\CentralProcessor\0' -ErrorAction Stop
} $null

$logicalProcessorCount = Convert-NullableUInt32 ([Environment]::ProcessorCount)
$cpuLoadAverage = Invoke-LiveSection {
    Get-ProcessCpuSample $logicalProcessorCount 220
} $null

if ($null -eq $cpuLoadAverage) {
    $cpuLoadAverage = Invoke-LiveSection {
        $cpuLoads = @(Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop |
            Where-Object { $null -ne $_.LoadPercentage })

        if ($cpuLoads.Count -eq 0) {
            $null
        } else {
            ($cpuLoads | Measure-Object -Property LoadPercentage -Average).Average
        }
    } $null
}

$os = Invoke-LiveSection {
    Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop
} $null

$memoryTotal = if ($null -eq $os) { $null } else { Convert-NullableUInt64 ($os.TotalVisibleMemorySize * 1KB) }
$memoryFree = if ($null -eq $os) { $null } else { Convert-NullableUInt64 ($os.FreePhysicalMemory * 1KB) }
$memoryUsed = if ($null -eq $memoryTotal -or $null -eq $memoryFree) { $null } else { [UInt64]($memoryTotal - $memoryFree) }

$volume = Invoke-LiveSection {
    [System.IO.DriveInfo]::GetDrives() |
        Where-Object { $_.DriveType -eq [System.IO.DriveType]::Fixed -and $_.IsReady } |
        Sort-Object -Property Name |
        Select-Object -First 1
} $null

$volumeTotal = if ($null -eq $volume) { $null } else { Convert-NullableUInt64 $volume.TotalSize }
$volumeFree = if ($null -eq $volume) { $null } else { Convert-NullableUInt64 $volume.AvailableFreeSpace }

$processIo = @(Invoke-LiveSection {
    Get-CimInstance -ClassName Win32_Process -ErrorAction Stop |
        Where-Object { $null -ne $_ }
} @())

$diskTransferTotal = if ($processIo.Count -eq 0) {
    $null
} else {
    $readTransfer = ($processIo | Measure-Object -Property ReadTransferCount -Sum).Sum
    $writeTransfer = ($processIo | Measure-Object -Property WriteTransferCount -Sum).Sum
    if ($null -eq $readTransfer) { $readTransfer = 0 }
    if ($null -eq $writeTransfer) { $writeTransfer = 0 }
    [UInt64]([UInt64]$readTransfer + [UInt64]$writeTransfer)
}

$networkStats = @(Invoke-LiveSection {
    Get-NetAdapterStatistics -ErrorAction Stop |
        Where-Object { $null -ne $_ }
} @())

$networkBytesTotal = if ($networkStats.Count -eq 0) {
    $null
} else {
    $received = ($networkStats | Measure-Object -Property ReceivedBytes -Sum).Sum
    $sent = ($networkStats | Measure-Object -Property SentBytes -Sum).Sum
    [UInt64]($received + $sent)
}

$activeAdapters = @(Invoke-LiveSection {
    Get-CimInstance -ClassName Win32_NetworkAdapter -Filter "NetConnectionStatus=2" -ErrorAction Stop |
        Where-Object { $_.PhysicalAdapter -eq $true }
} @())

$primaryAdapter = @($activeAdapters | Sort-Object -Property Speed -Descending | Select-Object -First 1)[0]

[ordered]@{
    collectedAtUtc = [DateTime]::UtcNow.ToString("o")
    source = "process CPU sample + registry CPU metadata + DriveInfo + Win32_Process + adapter statistics"
    cpu = [ordered]@{
        usagePercent = Convert-NullableDouble $cpuLoadAverage
        logicalProcessors = $logicalProcessorCount
        maxClockMhz = if ($null -eq $cpuRegistry) { $null } else { Convert-NullableUInt32 $cpuRegistry.'~MHz' }
        name = if ($null -eq $cpuRegistry) { $null } else { Convert-NullableString $cpuRegistry.ProcessorNameString }
    }
    memory = [ordered]@{
        totalBytes = $memoryTotal
        freeBytes = $memoryFree
        usedBytes = $memoryUsed
        usedPercent = Get-UsedPercent $memoryTotal $memoryFree
    }
    disk = [ordered]@{
        bytesPerSecond = $null
        totalBytes = Convert-NullableUInt64 $diskTransferTotal
        usedPercent = Get-UsedPercent $volumeTotal $volumeFree
        primaryVolume = if ($null -eq $volume) { $null } else { Convert-NullableString ([string]$volume.Name).TrimEnd('\') }
        health = $null
    }
    network = [ordered]@{
        bytesPerSecond = $null
        totalBytes = Convert-NullableUInt64 $networkBytesTotal
        linkSpeedBitsPerSecond = if ($null -eq $primaryAdapter) { $null } else { Convert-NullableUInt64 $primaryAdapter.Speed }
        activeAdapters = [UInt32]$activeAdapters.Count
        adapterName = if ($null -eq $primaryAdapter) { $null } else { Convert-NullableString $primaryAdapter.Name }
    }
} | ConvertTo-Json -Depth 5 -Compress
"#;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SecurityStatusRequest {
    include_allowlist: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SecurityStatusResponse {
    command_id: String,
    requester: String,
    deny_by_default: bool,
    allows_elevation: bool,
    allowlisted_commands: Vec<AllowlistedCommandResponse>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AllowlistedCommandResponse {
    command_id: String,
    payload_kind: String,
    risk: String,
    allows_elevation: bool,
    audit_denials: bool,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IpcErrorResponse {
    reason: String,
    message: String,
    command_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SystemScanRequest {
    requester: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SystemScanResponse {
    command_id: String,
    requester: String,
    report: SystemScanReport,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LiveResourceSnapshotRequest {
    requester: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveResourceSnapshotResponse {
    command_id: String,
    requester: String,
    collected_at_utc: String,
    source: String,
    cpu: LiveCpuSnapshot,
    memory: LiveMemorySnapshot,
    disk: LiveDiskSnapshot,
    network: LiveNetworkSnapshot,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LiveResourceSnapshotPayload {
    collected_at_utc: String,
    source: String,
    cpu: LiveCpuSnapshot,
    memory: LiveMemorySnapshot,
    disk: LiveDiskSnapshot,
    network: LiveNetworkSnapshot,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LiveCpuSnapshot {
    usage_percent: Option<f64>,
    logical_processors: Option<u32>,
    max_clock_mhz: Option<u32>,
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LiveMemorySnapshot {
    total_bytes: Option<u64>,
    free_bytes: Option<u64>,
    used_bytes: Option<u64>,
    used_percent: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LiveDiskSnapshot {
    bytes_per_second: Option<u64>,
    total_bytes: Option<u64>,
    used_percent: Option<f64>,
    primary_volume: Option<String>,
    health: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LiveNetworkSnapshot {
    bytes_per_second: Option<u64>,
    total_bytes: Option<u64>,
    link_speed_bits_per_second: Option<u64>,
    active_adapters: u32,
    adapter_name: Option<String>,
}

#[tauri::command]
pub(crate) fn get_ipc_security_status(
    requester: String,
    payload: SecurityStatusRequest,
) -> Result<SecurityStatusResponse, IpcErrorResponse> {
    let request = RawIpcRequest::security_status(
        requester,
        SecurityStatusPayload {
            include_allowlist: payload.include_allowlist,
        },
    )
    .map_err(IpcErrorResponse::from)?;

    let validated = validate_ipc_request(request).map_err(IpcErrorResponse::from)?;
    let allows_elevation = validated.policy().allows_elevation;
    let include_allowlist = match validated.payload() {
        IpcCommandPayload::SecurityStatus(payload) => payload.include_allowlist,
        IpcCommandPayload::Empty => false,
    };

    Ok(SecurityStatusResponse {
        command_id: validated.command_id().as_str().to_owned(),
        requester: validated.requester().as_str().to_owned(),
        deny_by_default: true,
        allows_elevation,
        allowlisted_commands: include_allowlist
            .then(|| {
                ipc_allowlist()
                    .iter()
                    .copied()
                    .map(AllowlistedCommandResponse::from)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

#[tauri::command]
pub(crate) async fn get_live_resource_snapshot(
    payload: LiveResourceSnapshotRequest,
) -> Result<LiveResourceSnapshotResponse, IpcErrorResponse> {
    let requester = validate_live_resource_snapshot_requester(payload.requester)?;
    let snapshot = tauri::async_runtime::spawn_blocking(collect_live_resource_snapshot)
        .await
        .map_err(|error| {
            live_resource_snapshot_error(
                "live_resource_snapshot_failed",
                format!("Read-only resource counter worker failed: {error}"),
            )
        })??;

    Ok(live_resource_snapshot_response(&requester, snapshot))
}

fn live_resource_snapshot_response(
    requester: &IpcRequester,
    snapshot: LiveResourceSnapshotPayload,
) -> LiveResourceSnapshotResponse {
    LiveResourceSnapshotResponse {
        command_id: LIVE_RESOURCE_SNAPSHOT_COMMAND_ID.to_owned(),
        requester: requester.as_str().to_owned(),
        collected_at_utc: snapshot.collected_at_utc,
        source: snapshot.source,
        cpu: snapshot.cpu,
        memory: snapshot.memory,
        disk: snapshot.disk,
        network: snapshot.network,
    }
}

#[tauri::command]
pub(crate) fn run_read_only_system_scan(
    payload: SystemScanRequest,
) -> Result<SystemScanResponse, IpcErrorResponse> {
    let requester = validate_system_scan_requester(payload.requester)?;
    let report = scan_system().map_err(IpcErrorResponse::from)?;

    Ok(system_scan_response(&requester, report))
}

fn validate_system_scan_requester(requester: String) -> Result<IpcRequester, IpcErrorResponse> {
    IpcRequester::new(requester).map_err(|denial| IpcErrorResponse {
        reason: denial.reason().as_str().to_owned(),
        message: denial.message().to_owned(),
        command_id: Some(SYSTEM_SCAN_COMMAND_ID.to_owned()),
    })
}

fn validate_live_resource_snapshot_requester(
    requester: String,
) -> Result<IpcRequester, IpcErrorResponse> {
    IpcRequester::new(requester).map_err(|denial| IpcErrorResponse {
        reason: denial.reason().as_str().to_owned(),
        message: denial.message().to_owned(),
        command_id: Some(LIVE_RESOURCE_SNAPSHOT_COMMAND_ID.to_owned()),
    })
}

#[cfg(windows)]
fn collect_live_resource_snapshot() -> Result<LiveResourceSnapshotPayload, IpcErrorResponse> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            LIVE_RESOURCE_SNAPSHOT_SCRIPT,
        ])
        .output()
        .map_err(|error| {
            live_resource_snapshot_error(
                "live_resource_snapshot_failed",
                format!("Failed to start read-only resource counter collection: {error}"),
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let detail = if stderr.is_empty() { stdout } else { stderr };

        return Err(live_resource_snapshot_error(
            "live_resource_snapshot_failed",
            if detail.is_empty() {
                "Read-only resource counter collection failed without diagnostic output.".to_owned()
            } else {
                detail
            },
        ));
    }

    serde_json::from_slice(&output.stdout).map_err(|error| {
        live_resource_snapshot_error(
            "live_resource_snapshot_parse_failed",
            format!("Read-only resource counters returned an invalid payload: {error}"),
        )
    })
}

#[cfg(not(windows))]
fn collect_live_resource_snapshot() -> Result<LiveResourceSnapshotPayload, IpcErrorResponse> {
    Err(live_resource_snapshot_error(
        "unsupported_platform",
        "Live resource counters are only available in the Windows desktop shell.",
    ))
}

fn live_resource_snapshot_error(
    reason: &'static str,
    message: impl Into<String>,
) -> IpcErrorResponse {
    IpcErrorResponse {
        reason: reason.to_owned(),
        message: message.into(),
        command_id: Some(LIVE_RESOURCE_SNAPSHOT_COMMAND_ID.to_owned()),
    }
}

fn system_scan_response(requester: &IpcRequester, report: SystemScanReport) -> SystemScanResponse {
    SystemScanResponse {
        command_id: SYSTEM_SCAN_COMMAND_ID.to_owned(),
        requester: requester.as_str().to_owned(),
        report,
    }
}

impl From<optimizer_core::ipc::IpcCommandPolicy> for AllowlistedCommandResponse {
    fn from(policy: optimizer_core::ipc::IpcCommandPolicy) -> Self {
        Self {
            command_id: policy.command_id.as_str().to_owned(),
            payload_kind: policy.payload_kind.as_str().to_owned(),
            risk: policy.risk.as_str().to_owned(),
            allows_elevation: policy.allows_elevation,
            audit_denials: policy.audit_denials,
        }
    }
}

impl From<IpcDenial> for IpcErrorResponse {
    fn from(denial: IpcDenial) -> Self {
        Self {
            reason: denial.reason().as_str().to_owned(),
            message: denial.message().to_owned(),
            command_id: denial.command_id().map(ToOwned::to_owned),
        }
    }
}

impl From<SystemScanError> for IpcErrorResponse {
    fn from(error: SystemScanError) -> Self {
        Self {
            reason: error.reason().as_str().to_owned(),
            message: error.message().to_owned(),
            command_id: Some(SYSTEM_SCAN_COMMAND_ID.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optimizer_core::ipc::{IpcCommandId, IpcRisk};
    use windows_api::{parse_system_scan_report, SystemScanMode};

    const SYSTEM_SCAN_FIXTURE: &str =
        include_str!("../../../../crates/windows-api/tests/fixtures/system_scan.json");

    #[test]
    fn security_status_command_returns_allowlist_when_requested() {
        let response = get_ipc_security_status(
            "main-window".to_owned(),
            SecurityStatusRequest {
                include_allowlist: true,
            },
        )
        .expect("read-only IPC status should be available");

        assert_eq!(response.command_id, IpcCommandId::SecurityStatus.as_str());
        assert_eq!(response.requester, "main-window");
        assert!(response.deny_by_default);
        assert!(!response.allows_elevation);
        assert_eq!(response.allowlisted_commands.len(), 1);
        assert_eq!(
            response.allowlisted_commands[0].risk,
            IpcRisk::ReadOnly.as_str()
        );
    }

    #[test]
    fn security_status_command_can_omit_allowlist_details() {
        let response = get_ipc_security_status(
            "main-window".to_owned(),
            SecurityStatusRequest {
                include_allowlist: false,
            },
        )
        .expect("read-only IPC status should be available");

        assert!(response.allowlisted_commands.is_empty());
    }

    #[test]
    fn security_status_command_rejects_invalid_requester() {
        let error = get_ipc_security_status(
            "main window; shell".to_owned(),
            SecurityStatusRequest {
                include_allowlist: true,
            },
        )
        .expect_err("invalid requester should be denied");

        assert_eq!(error.reason, "invalid_requester");
        assert_eq!(error.command_id, None);
    }

    #[test]
    fn system_scan_binding_maps_typed_fixture_report() {
        let requester =
            IpcRequester::new("main-window").expect("fixture requester should be valid");
        let report =
            parse_system_scan_report(SYSTEM_SCAN_FIXTURE).expect("fixture report should parse");
        let response = system_scan_response(&requester, report);

        assert_eq!(response.command_id, SYSTEM_SCAN_COMMAND_ID);
        assert_eq!(response.requester, "main-window");
        assert_eq!(response.report.scan_mode, SystemScanMode::ReadOnly);
        assert!(response.report.covers_t040_inventory());
    }

    #[test]
    fn system_scan_command_rejects_invalid_requester_before_scan() {
        let error = run_read_only_system_scan(SystemScanRequest {
            requester: "main window; shell".to_owned(),
        })
        .expect_err("invalid requester should be denied before live scan");

        assert_eq!(error.reason, "invalid_requester");
        assert_eq!(error.command_id, Some(SYSTEM_SCAN_COMMAND_ID.to_owned()));
    }

    #[test]
    fn live_resource_snapshot_rejects_invalid_requester_before_collecting_counters() {
        let error = tauri::async_runtime::block_on(get_live_resource_snapshot(
            LiveResourceSnapshotRequest {
                requester: "main window; shell".to_owned(),
            },
        ))
        .expect_err("invalid requester should be denied before live counters run");

        assert_eq!(error.reason, "invalid_requester");
        assert_eq!(
            error.command_id,
            Some(LIVE_RESOURCE_SNAPSHOT_COMMAND_ID.to_owned())
        );
    }

    #[test]
    fn live_resource_snapshot_response_maps_payload() {
        let requester =
            IpcRequester::new("main-window").expect("fixture requester should be valid");
        let snapshot = LiveResourceSnapshotPayload {
            collected_at_utc: "2026-05-03T19:42:00Z".to_owned(),
            source: "fixture".to_owned(),
            cpu: LiveCpuSnapshot {
                usage_percent: Some(12.4),
                logical_processors: Some(16),
                max_clock_mhz: Some(4700),
                name: Some("Fixture CPU".to_owned()),
            },
            memory: LiveMemorySnapshot {
                free_bytes: Some(8),
                total_bytes: Some(16),
                used_bytes: Some(8),
                used_percent: Some(50.0),
            },
            disk: LiveDiskSnapshot {
                bytes_per_second: Some(1024),
                health: Some("Healthy".to_owned()),
                primary_volume: Some("C:".to_owned()),
                total_bytes: Some(2048),
                used_percent: Some(40.0),
            },
            network: LiveNetworkSnapshot {
                active_adapters: 1,
                adapter_name: Some("Ethernet".to_owned()),
                bytes_per_second: Some(512),
                link_speed_bits_per_second: Some(1_000_000_000),
                total_bytes: Some(4096),
            },
        };
        let response = live_resource_snapshot_response(&requester, snapshot);

        assert_eq!(response.command_id, LIVE_RESOURCE_SNAPSHOT_COMMAND_ID);
        assert_eq!(response.requester, "main-window");
        assert_eq!(response.cpu.usage_percent, Some(12.4));
        assert_eq!(response.network.bytes_per_second, Some(512));
    }
}
