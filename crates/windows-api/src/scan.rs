//! Read-only Windows system scan adapter.

use std::fmt;

#[cfg(windows)]
use std::process::Command;

use serde::{Deserialize, Serialize};

#[cfg(windows)]
const LIVE_SCAN_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$collectionErrors = @()

function Invoke-ScanSection($section, [scriptblock]$body, $fallback) {
    try {
        & $body
    } catch {
        $script:collectionErrors += [ordered]@{
            section = $section
            message = $_.Exception.Message
        }
        $fallback
    }
}

function Convert-NullableUInt64($value) {
    if ($null -eq $value) {
        $null
    } else {
        [UInt64]$value
    }
}

function Convert-NullableUInt32($value) {
    if ($null -eq $value) {
        $null
    } else {
        [UInt32]$value
    }
}

function Convert-NullableBool($value) {
    if ($null -eq $value) {
        $null
    } else {
        [bool]$value
    }
}

function Convert-NullableEnabledBool($value) {
    if ($null -eq $value) {
        $null
    } else {
        $text = ([string]$value).Trim()
        if ($text -match '^(Enabled|Enable|True|1)$') {
            $true
        } elseif ($text -match '^(Disabled|Disable|False|0)$') {
            $false
        } else {
            $null
        }
    }
}

function Get-PropertyValue($props, [string]$name) {
    if ($null -eq $props) {
        $null
    } else {
        $property = $props.PSObject.Properties[$name]
        if ($null -eq $property) { $null } else { $property.Value }
    }
}

function Get-FirstPropertyValue($props, [string[]]$names) {
    foreach ($name in $names) {
        $value = Get-PropertyValue $props $name
        if ($null -ne $value) {
            return $value
        }
    }

    $null
}

function Measure-CleanupCandidate([string]$logicalTarget, [string]$path, [string]$kind) {
    $expanded = [Environment]::ExpandEnvironmentVariables($path)
    $exists = Test-Path -LiteralPath $expanded
    $fileCount = 0
    $bytes = [UInt64]0

    if ($exists) {
        $measure = Get-ChildItem -LiteralPath $expanded -Force -Recurse -File -ErrorAction SilentlyContinue |
            Measure-Object -Property Length -Sum
        $fileCount = if ($null -eq $measure.Count) { 0 } else { [UInt32]$measure.Count }
        $bytes = if ($null -eq $measure.Sum) { [UInt64]0 } else { [UInt64]$measure.Sum }
    }

    [ordered]@{
        target = $logicalTarget
        path = $expanded
        kind = $kind
        reclaimableBytes = $bytes
        fileCount = $fileCount
        safeToPreview = $exists
    }
}

$os = Invoke-ScanSection "os" {
    $item = Get-CimInstance -ClassName Win32_OperatingSystem
    [ordered]@{
        caption = [string]$item.Caption
        version = [string]$item.Version
        buildNumber = [string]$item.BuildNumber
        edition = [string]$item.OperatingSystemSKU
        architecture = [string]$item.OSArchitecture
        lastBootUpTime = if ($null -eq $item.LastBootUpTime) { $null } else { [string]$item.LastBootUpTime.ToUniversalTime().ToString("o") }
        installDate = if ($null -eq $item.InstallDate) { $null } else { [string]$item.InstallDate.ToUniversalTime().ToString("o") }
        totalVisibleMemoryBytes = Convert-NullableUInt64 ($item.TotalVisibleMemorySize * 1KB)
        freePhysicalMemoryBytes = Convert-NullableUInt64 ($item.FreePhysicalMemory * 1KB)
    }
} ([ordered]@{})

$cpus = @(Invoke-ScanSection "cpu" {
    @(Get-CimInstance -ClassName Win32_Processor | ForEach-Object {
        [ordered]@{
            name = [string]$_.Name
            manufacturer = [string]$_.Manufacturer
            architecture = [string]$_.Architecture
            physicalCores = Convert-NullableUInt32 $_.NumberOfCores
            logicalProcessors = Convert-NullableUInt32 $_.NumberOfLogicalProcessors
            maxClockMhz = Convert-NullableUInt32 $_.MaxClockSpeed
        }
    })
} @())

$memoryModules = @(Invoke-ScanSection "memory.modules" {
    @(Get-CimInstance -ClassName Win32_PhysicalMemory | ForEach-Object {
        [ordered]@{
            manufacturer = if ($null -eq $_.Manufacturer) { $null } else { [string]$_.Manufacturer.Trim() }
            partNumber = if ($null -eq $_.PartNumber) { $null } else { [string]$_.PartNumber.Trim() }
            capacityBytes = Convert-NullableUInt64 $_.Capacity
            speedMhz = Convert-NullableUInt32 $_.Speed
        }
    })
} @())

$gpus = @(Invoke-ScanSection "gpu" {
    @(Get-CimInstance -ClassName Win32_VideoController | ForEach-Object {
        [ordered]@{
            name = [string]$_.Name
            driverVersion = if ($null -eq $_.DriverVersion) { $null } else { [string]$_.DriverVersion }
            videoProcessor = if ($null -eq $_.VideoProcessor) { $null } else { [string]$_.VideoProcessor }
            adapterRamBytes = Convert-NullableUInt64 $_.AdapterRAM
            pnpDeviceId = if ($null -eq $_.PNPDeviceID) { $null } else { [string]$_.PNPDeviceID }
        }
    })
} @())

$volumes = @(Invoke-ScanSection "storage.volumes" {
    @(Get-CimInstance -ClassName Win32_LogicalDisk -Filter "DriveType=3" | ForEach-Object {
        [ordered]@{
            deviceId = [string]$_.DeviceID
            volumeName = if ($null -eq $_.VolumeName) { $null } else { [string]$_.VolumeName }
            fileSystem = if ($null -eq $_.FileSystem) { $null } else { [string]$_.FileSystem }
            totalBytes = Convert-NullableUInt64 $_.Size
            freeBytes = Convert-NullableUInt64 $_.FreeSpace
            driveType = Convert-NullableUInt32 $_.DriveType
        }
    })
} @())

$physicalDisks = @(Invoke-ScanSection "storage.physical" {
    @(Get-PhysicalDisk | ForEach-Object {
        [ordered]@{
            friendlyName = [string]$_.FriendlyName
            mediaType = if ($null -eq $_.MediaType) { $null } else { [string]$_.MediaType }
            busType = if ($null -eq $_.BusType) { $null } else { [string]$_.BusType }
            healthStatus = if ($null -eq $_.HealthStatus) { $null } else { [string]$_.HealthStatus }
            sizeBytes = Convert-NullableUInt64 $_.Size
        }
    })
} @())

$storageCleanup = Invoke-ScanSection "storage.cleanup" {
    [ordered]@{
        candidates = @(
            Measure-CleanupCandidate "user-temp" "%TEMP%" "user_temp"
            Measure-CleanupCandidate "windows-temp" "$env:SystemRoot\Temp" "windows_temp"
            Measure-CleanupCandidate "directx-shader-cache" "$env:LOCALAPPDATA\D3DSCache" "shader_cache"
        )
        excludedPatterns = @(
            "Downloads"
            "Documents"
            "Desktop"
            "game install folders"
            "PUBG game content"
            "in-use files"
        )
    }
} ([ordered]@{
    candidates = @()
    excludedPatterns = @()
})

$storageSense = Invoke-ScanSection "storage.sense" {
    $policyPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\StorageSense\Parameters\StoragePolicy"
    $props = if (Test-Path $policyPath) { Get-ItemProperty -Path $policyPath } else { $null }
    $enabledRaw = Get-PropertyValue $props "01"
    [ordered]@{
        enabled = if ($null -eq $enabledRaw) { $null } else { [UInt32]$enabledRaw -ne 0 }
        cadenceDays = Convert-NullableUInt32 (Get-PropertyValue $props "2048")
        recycleBinCleanupDays = Convert-NullableUInt32 (Get-PropertyValue $props "256")
        downloadsCleanupDays = Convert-NullableUInt32 (Get-PropertyValue $props "512")
        source = "HKCU StoragePolicy"
    }
} ([ordered]@{
    enabled = $null
    cadenceDays = $null
    recycleBinCleanupDays = $null
    downloadsCleanupDays = $null
    source = "HKCU StoragePolicy"
})

$trim = Invoke-ScanSection "storage.trim" {
    $output = (& "$env:SystemRoot\System32\fsutil.exe" behavior query DisableDeleteNotify) -join "`n"
    $ntfs = $null
    $refs = $null
    if ($output -match "NTFS DisableDeleteNotify\s*=\s*(\d+)") {
        $ntfs = [UInt32]$Matches[1]
    }
    if ($output -match "ReFS DisableDeleteNotify\s*=\s*(\d+)") {
        $refs = [UInt32]$Matches[1]
    }

    [ordered]@{
        ntfsDisableDeleteNotify = $ntfs
        refsDisableDeleteNotify = $refs
        optimizeVolumeAvailable = [bool](Get-Command Optimize-Volume -ErrorAction SilentlyContinue)
        source = "fsutil behavior query DisableDeleteNotify"
    }
} ([ordered]@{
    ntfsDisableDeleteNotify = $null
    refsDisableDeleteNotify = $null
    optimizeVolumeAvailable = $false
    source = "fsutil behavior query DisableDeleteNotify"
})

$directStorage = Invoke-ScanSection "storage.direct_storage" {
    $buildNumber = 0
    $buildParsed = [Int32]::TryParse([string]$os.buildNumber, [ref]$buildNumber)
    $nvmePresent = @($physicalDisks | Where-Object { [string]$_["busType"] -match "NVMe" }).Count -gt 0
    [ordered]@{
        osSupported = if ($buildParsed) { $buildNumber -ge 19041 } else { $null }
        nvmePresent = $nvmePresent
        gpuDecompressionSupported = $null
        gameVolumeBusType = $null
        source = "scan-derived DirectStorage readiness"
    }
} ([ordered]@{
    osSupported = $null
    nvmePresent = $null
    gpuDecompressionSupported = $null
    gameVolumeBusType = $null
    source = "scan-derived DirectStorage readiness"
})

$networkAdapters = @(Invoke-ScanSection "network.adapters" {
    @(Get-CimInstance -ClassName Win32_NetworkAdapter -Filter "PhysicalAdapter=True" | ForEach-Object {
        $adapterAlias = if ($null -eq $_.NetConnectionID) { $null } else { [string]$_.NetConnectionID }
        $powerManagement = [ordered]@{
            allowComputerToTurnOffDevice = $null
            source = "Get-NetAdapterPowerManagement unavailable"
        }
        $advancedProperties = @()

        if ($null -ne $adapterAlias -and $adapterAlias.Length -gt 0) {
            try {
                $pm = Get-NetAdapterPowerManagement -Name $adapterAlias -ErrorAction Stop
                $powerManagement = [ordered]@{
                    allowComputerToTurnOffDevice = Convert-NullableEnabledBool (Get-PropertyValue $pm "AllowComputerToTurnOffDevice")
                    source = "Get-NetAdapterPowerManagement"
                }
            } catch {
                $powerManagement = [ordered]@{
                    allowComputerToTurnOffDevice = $null
                    source = "Get-NetAdapterPowerManagement unavailable"
                }
            }

            try {
                $advancedProperties = @(Get-NetAdapterAdvancedProperty -Name $adapterAlias -ErrorAction Stop | ForEach-Object {
                    $registryValue = Get-PropertyValue $_ "RegistryValue"
                    [ordered]@{
                        displayName = if ($null -eq $_.DisplayName) { [string]$_.Name } else { [string]$_.DisplayName }
                        displayValue = if ($null -eq $_.DisplayValue) { $null } else { [string]$_.DisplayValue }
                        registryKeyword = if ($null -eq $_.RegistryKeyword) { $null } else { [string]$_.RegistryKeyword }
                        registryValue = if ($null -eq $registryValue) { $null } elseif ($registryValue -is [array]) { [string]::Join(",", @($registryValue)) } else { [string]$registryValue }
                    }
                })
            } catch {
                $advancedProperties = @()
            }
        }

        [ordered]@{
            name = [string]$_.Name
            adapterType = if ($null -eq $_.AdapterType) { $null } else { [string]$_.AdapterType }
            macAddress = if ($null -eq $_.MACAddress) { $null } else { [string]$_.MACAddress }
            netConnectionId = $adapterAlias
            netConnectionStatus = Convert-NullableUInt32 $_.NetConnectionStatus
            speedBitsPerSecond = Convert-NullableUInt64 $_.Speed
            powerManagement = $powerManagement
            advancedProperties = @($advancedProperties)
        }
    })
} @())

$services = @(Invoke-ScanSection "services" {
    @(Get-CimInstance -ClassName Win32_Service | ForEach-Object {
        [ordered]@{
            name = [string]$_.Name
            displayName = if ($null -eq $_.DisplayName) { $null } else { [string]$_.DisplayName }
            state = if ($null -eq $_.State) { $null } else { [string]$_.State }
            startMode = if ($null -eq $_.StartMode) { $null } else { [string]$_.StartMode }
        }
    })
} @())

$scheduledTasks = @(Invoke-ScanSection "scheduled_tasks" {
    @(Get-ScheduledTask | ForEach-Object {
        $taskInfo = $null
        try {
            $taskInfo = $_ | Get-ScheduledTaskInfo -ErrorAction Stop
        } catch {
            $taskInfo = $null
        }
        $nextRunTime = if ($null -eq $taskInfo -or $taskInfo.NextRunTime -eq [DateTime]::MinValue) {
            $null
        } else {
            [string]$taskInfo.NextRunTime.ToString("o")
        }
        $lastRunTime = if ($null -eq $taskInfo -or $taskInfo.LastRunTime -eq [DateTime]::MinValue) {
            $null
        } else {
            [string]$taskInfo.LastRunTime.ToString("o")
        }
        [ordered]@{
            taskName = [string]$_.TaskName
            taskPath = [string]$_.TaskPath
            state = if ($null -eq $_.State) { $null } else { [string]$_.State }
            nextRunTime = $nextRunTime
            lastRunTime = $lastRunTime
        }
    })
} @())

$startupApps = @(Invoke-ScanSection "startup_apps" {
    @(Get-CimInstance -ClassName Win32_StartupCommand | ForEach-Object {
        [ordered]@{
            name = [string]$_.Name
            command = if ($null -eq $_.Command) { $null } else { [string]$_.Command }
            location = if ($null -eq $_.Location) { $null } else { [string]$_.Location }
            user = if ($null -eq $_.User) { $null } else { [string]$_.User }
            enabled = $null
            startupImpact = $null
        }
    })
} @())

$backgroundApps = @(Invoke-ScanSection "background_apps" {
    $root = "HKCU:\Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications"
    if (Test-Path $root) {
        @(Get-ChildItem -Path $root | ForEach-Object {
            $props = Get-ItemProperty -Path $_.PSPath
            $disabled = if ($null -eq $props.Disabled) { $null } else { [bool]$props.Disabled }
            $disabledByUser = if ($null -eq $props.DisabledByUser) { $null } else { [bool]$props.DisabledByUser }
            [ordered]@{
                appId = [string]$_.PSChildName
                displayName = if ($null -eq $props.DisplayName) { $null } else { [string]$props.DisplayName }
                enabled = if ($null -eq $disabled -and $null -eq $disabledByUser) { $null } else { -not (($disabled -eq $true) -or ($disabledByUser -eq $true)) }
                disabled = $disabled
                disabledByUser = $disabledByUser
                activity = $null
            }
        })
    } else {
        @()
    }
} @())

$power = Invoke-ScanSection "power.active_plan" {
    $line = (& "$env:SystemRoot\System32\powercfg.exe" /getactivescheme) -join "`n"
    $guid = $null
    $name = $null
    if ($line -match "([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})") {
        $guid = $Matches[1]
    }
    $firstParen = $line.IndexOf("(")
    $lastParen = $line.LastIndexOf(")")
    if ($firstParen -ge 0 -and $lastParen -gt $firstParen) {
        $name = $line.Substring($firstParen + 1, $lastParen - $firstParen - 1)
    }
    [ordered]@{
        activeSchemeGuid = $guid
        activeSchemeName = $name
        source = "powercfg /getactivescheme"
    }
} ([ordered]@{
    activeSchemeGuid = $null
    activeSchemeName = $null
    source = "powercfg /getactivescheme"
})

$scheduler = Invoke-ScanSection "scheduler.registry" {
    $mmcssPath = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile"
    $priorityPath = "HKLM:\SYSTEM\CurrentControlSet\Control\PriorityControl"
    $mmcssProps = Get-ItemProperty -Path $mmcssPath -ErrorAction SilentlyContinue
    $priorityProps = Get-ItemProperty -Path $priorityPath -ErrorAction SilentlyContinue
    [ordered]@{
        mmcssSystemResponsiveness = Convert-NullableUInt32 (Get-PropertyValue $mmcssProps "SystemResponsiveness")
        win32PrioritySeparation = Convert-NullableUInt32 (Get-PropertyValue $priorityProps "Win32PrioritySeparation")
        source = "HKLM scheduler registry"
    }
} ([ordered]@{
    mmcssSystemResponsiveness = $null
    win32PrioritySeparation = $null
    source = "HKLM scheduler registry"
})

$graphics = Invoke-ScanSection "graphics.settings" {
    $graphicsDriversPath = "HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers"
    $graphicsSettingsPath = "HKCU:\Software\Microsoft\DirectX\GraphicsSettings"
    $userGpuPreferencesPath = "HKCU:\Software\Microsoft\DirectX\UserGpuPreferences"
    $graphicsDriversProps = Get-ItemProperty -Path $graphicsDriversPath -ErrorAction SilentlyContinue
    $graphicsSettingsProps = Get-ItemProperty -Path $graphicsSettingsPath -ErrorAction SilentlyContinue
    $hagsRaw = Get-PropertyValue $graphicsDriversProps "HwSchMode"
    $windowedRaw = Get-FirstPropertyValue $graphicsSettingsProps @(
        "SwapEffectUpgradeEnable",
        "SwapEffectUpgrade"
    )
    $vrrRaw = Get-FirstPropertyValue $graphicsSettingsProps @(
        "VariableRefreshRate",
        "VRROptimizeEnable",
        "VRREnable"
    )
    $appPreferences = @()

    if (Test-Path $userGpuPreferencesPath) {
        $preferenceProps = Get-ItemProperty -Path $userGpuPreferencesPath -ErrorAction SilentlyContinue
        if ($null -ne $preferenceProps) {
            $appPreferences = @($preferenceProps.PSObject.Properties | Where-Object {
                $_.Name -notlike "PS*" -and $null -ne $_.Value
            } | ForEach-Object {
                [ordered]@{
                    executablePath = [string]$_.Name
                    preference = [string]$_.Value
                }
            })
        }
    }

    $buildNumber = 0
    $buildParsed = [Int32]::TryParse([string]$os.buildNumber, [ref]$buildNumber)
    $windowedSupported = if ($buildParsed) { $buildNumber -ge 22000 } else { $null }
    $hagsSupported = if ($null -ne $hagsRaw) { $true } else { $null }
    $vrrSupported = if ($null -ne $vrrRaw) { $true } else { $null }
    $highPerformanceGpuAvailable = @($gpus).Count -gt 1

    [ordered]@{
        hags = [ordered]@{
            value = Convert-NullableUInt32 $hagsRaw
            supported = $hagsSupported
            source = "HKLM GraphicsDrivers HwSchMode"
        }
        windowedOptimizations = [ordered]@{
            value = Convert-NullableUInt32 $windowedRaw
            supported = $windowedSupported
            source = "HKCU DirectX GraphicsSettings SwapEffectUpgradeEnable"
        }
        variableRefreshRate = [ordered]@{
            value = Convert-NullableUInt32 $vrrRaw
            supported = $vrrSupported
            source = "HKCU DirectX GraphicsSettings VRR"
        }
        highPerformanceGpuAvailable = $highPerformanceGpuAvailable
        appPreferences = @($appPreferences)
        source = "Windows Graphics settings registry read-only"
    }
} ([ordered]@{
    hags = [ordered]@{
        value = $null
        supported = $null
        source = "HKLM GraphicsDrivers HwSchMode"
    }
    windowedOptimizations = [ordered]@{
        value = $null
        supported = $null
        source = "HKCU DirectX GraphicsSettings SwapEffectUpgradeEnable"
    }
    variableRefreshRate = [ordered]@{
        value = $null
        supported = $null
        source = "HKCU DirectX GraphicsSettings VRR"
    }
    highPerformanceGpuAvailable = $null
    appPreferences = @()
    source = "Windows Graphics settings registry read-only"
})

$deviceGuard = Invoke-ScanSection "security.device_guard" {
    $dg = Get-CimInstance -Namespace "root\Microsoft\Windows\DeviceGuard" -ClassName Win32_DeviceGuard
    [ordered]@{
        virtualizationBasedSecurityStatus = Convert-NullableUInt32 $dg.VirtualizationBasedSecurityStatus
        securityServicesConfigured = @($dg.SecurityServicesConfigured | ForEach-Object { [UInt32]$_ })
        securityServicesRunning = @($dg.SecurityServicesRunning | ForEach-Object { [UInt32]$_ })
        usermodeCodeIntegrityPolicyEnforcementStatus = Convert-NullableUInt32 $dg.UsermodeCodeIntegrityPolicyEnforcementStatus
        codeIntegrityPolicyEnforcementStatus = Convert-NullableUInt32 $dg.CodeIntegrityPolicyEnforcementStatus
    }
} ([ordered]@{
    virtualizationBasedSecurityStatus = $null
    securityServicesConfigured = @()
    securityServicesRunning = @()
    usermodeCodeIntegrityPolicyEnforcementStatus = $null
    codeIntegrityPolicyEnforcementStatus = $null
})

$hvci = Invoke-ScanSection "security.hvci" {
    $path = "HKLM:\SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity"
    $props = Get-ItemProperty -Path $path
    [ordered]@{
        enabled = Convert-NullableUInt32 $props.Enabled
        locked = Convert-NullableUInt32 $props.Locked
    }
} ([ordered]@{
    enabled = $null
    locked = $null
})

$optionalFeatures = @(Invoke-ScanSection "security.optional_features" {
    $featureNames = @(
        "VirtualMachinePlatform"
        "Microsoft-Hyper-V-All"
        "Microsoft-Windows-Subsystem-Linux"
        "HypervisorPlatform"
    )

    @(Get-CimInstance -ClassName Win32_OptionalFeature | Where-Object {
        $featureNames -contains $_.Name
    } | ForEach-Object {
        [ordered]@{
            name = [string]$_.Name
            caption = if ($null -eq $_.Caption) { $null } else { [string]$_.Caption }
            installState = Convert-NullableUInt32 $_.InstallState
        }
    })
} @())

$defender = Invoke-ScanSection "security.defender" {
    $status = Get-MpComputerStatus
    $prefs = $null
    try {
        $prefs = Get-MpPreference -ErrorAction Stop
    } catch {
        $prefs = $null
    }
    [ordered]@{
        antivirusEnabled = [bool]$status.AntivirusEnabled
        realTimeProtectionEnabled = [bool]$status.RealTimeProtectionEnabled
        tamperProtected = if ($null -eq $status.IsTamperProtected) { $null } else { [bool]$status.IsTamperProtected }
        antispywareEnabled = [bool]$status.AntispywareEnabled
        exclusionPaths = if ($null -eq $prefs) { @() } else { @($prefs.ExclusionPath | ForEach-Object { [string]$_ }) }
        scanScheduleDay = if ($null -eq $prefs) { $null } else { Convert-NullableUInt32 $prefs.ScanScheduleDay }
        scanScheduleTime = if ($null -eq $prefs -or $null -eq $prefs.ScanScheduleTime) { $null } else { [string]$prefs.ScanScheduleTime }
    }
} ([ordered]@{
    antivirusEnabled = $null
    realTimeProtectionEnabled = $null
    tamperProtected = $null
    antispywareEnabled = $null
    exclusionPaths = @()
    scanScheduleDay = $null
    scanScheduleTime = $null
})

$pendingFileRename = $false
$pendingFileRename = Invoke-ScanSection "reboot.pending_file_rename" {
    $value = Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager" -Name PendingFileRenameOperations -ErrorAction SilentlyContinue
    $null -ne $value.PendingFileRenameOperations
} $false

$rebootRequired = [ordered]@{
    componentBasedServicing = [bool](Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending")
    windowsUpdate = [bool](Test-Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired")
    pendingFileRename = [bool]$pendingFileRename
}

[ordered]@{
    schemaVersion = 1
    scanMode = "read_only"
    collectedAtUtc = [string](Get-Date).ToUniversalTime().ToString("o")
    os = $os
    memory = [ordered]@{
        totalVisibleMemoryBytes = $os.totalVisibleMemoryBytes
        freePhysicalMemoryBytes = $os.freePhysicalMemoryBytes
        modules = $memoryModules
    }
    cpus = $cpus
    gpus = $gpus
    storage = [ordered]@{
        volumes = $volumes
        physicalDisks = $physicalDisks
        cleanup = $storageCleanup
        storageSense = $storageSense
        trim = $trim
        directStorage = $directStorage
    }
    networkAdapters = $networkAdapters
    services = $services
    scheduledTasks = $scheduledTasks
    startupApps = $startupApps
    backgroundApps = $backgroundApps
    power = $power
    scheduler = $scheduler
    graphics = $graphics
    security = [ordered]@{
        deviceGuard = $deviceGuard
        hvci = $hvci
        optionalFeatures = $optionalFeatures
        defender = $defender
    }
    rebootRequired = $rebootRequired
    collectionErrors = @($collectionErrors)
} | ConvertTo-Json -Depth 8 -Compress
"#;

/// Runs the live read-only Windows system scan.
pub fn scan_system() -> Result<SystemScanReport, SystemScanError> {
    WindowsSystemScanner::new().scan()
}

/// Parses the JSON payload emitted by the PowerShell scan script.
pub fn parse_system_scan_report(payload: &str) -> Result<SystemScanReport, SystemScanError> {
    serde_json::from_str(payload).map_err(|source| SystemScanError::parse(source.to_string()))
}

/// Live scanner for the read-only system inventory.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WindowsSystemScanner;

impl WindowsSystemScanner {
    /// Creates a live scanner.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Collects the typed read-only system report.
    pub fn scan(self) -> Result<SystemScanReport, SystemScanError> {
        run_live_scan()
    }
}

/// A complete read-only system inventory report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemScanReport {
    /// Schema version of the report payload.
    pub schema_version: u16,
    /// Confirms the scan performed no mutation.
    pub scan_mode: SystemScanMode,
    /// UTC timestamp emitted by the scanner.
    pub collected_at_utc: String,
    /// Windows OS inventory.
    pub os: OsScan,
    /// RAM inventory.
    pub memory: MemoryScan,
    /// CPU inventory.
    pub cpus: Vec<CpuScanItem>,
    /// GPU inventory.
    pub gpus: Vec<GpuScanItem>,
    /// Storage inventory.
    pub storage: StorageScan,
    /// Physical network adapter inventory.
    pub network_adapters: Vec<NetworkAdapterScanItem>,
    /// Service inventory.
    pub services: Vec<ServiceScanItem>,
    /// Scheduled task inventory.
    pub scheduled_tasks: Vec<ScheduledTaskScanItem>,
    /// Startup app inventory.
    pub startup_apps: Vec<StartupAppScanItem>,
    /// Background app permission/activity inventory.
    #[serde(default)]
    pub background_apps: Vec<BackgroundAppScanItem>,
    /// Active power plan inventory.
    pub power: PowerPlanScan,
    /// Scheduler registry state used by Competitive scheduler planning.
    #[serde(default)]
    pub scheduler: SchedulerRegistryScan,
    /// Windows Graphics settings state used by graphics setting planning.
    #[serde(default)]
    pub graphics: GraphicsSettingsScan,
    /// VBS, HVCI, VMP, Hyper-V, and Defender read-only state.
    pub security: SecurityScan,
    /// Reboot-required markers.
    pub reboot_required: RebootRequiredScan,
    /// Non-fatal section collection errors.
    pub collection_errors: Vec<ScanCollectionError>,
}

impl SystemScanReport {
    /// Returns true when the scan covers all T040 inventory categories.
    #[must_use]
    pub fn covers_t040_inventory(&self) -> bool {
        self.schema_version == 1
            && self.scan_mode == SystemScanMode::ReadOnly
            && !self.os.caption.is_empty()
            && !self.os.version.is_empty()
            && !self.cpus.is_empty()
            && self.memory.total_visible_memory_bytes.is_some()
            && !self.gpus.is_empty()
            && !self.storage.volumes.is_empty()
            && !self.network_adapters.is_empty()
            && !self.services.is_empty()
            && !self.scheduled_tasks.is_empty()
            && !self.startup_apps.is_empty()
            && self.power.source == "powercfg /getactivescheme"
    }
}

/// Scan mode emitted by the inventory adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemScanMode {
    /// The scan only reads system state and performs no mutation.
    ReadOnly,
}

/// Windows OS inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OsScan {
    /// Marketing caption, such as Windows edition text.
    pub caption: String,
    /// Windows version string.
    pub version: String,
    /// Windows build number.
    pub build_number: String,
    /// Numeric Windows SKU when exposed by WMI.
    pub edition: Option<String>,
    /// OS architecture string.
    pub architecture: String,
    /// Last boot time in UTC ISO-8601 format when available.
    pub last_boot_up_time: Option<String>,
    /// Install date in UTC ISO-8601 format when available.
    pub install_date: Option<String>,
    /// Total visible memory reported by Windows.
    pub total_visible_memory_bytes: Option<u64>,
    /// Free physical memory reported by Windows.
    pub free_physical_memory_bytes: Option<u64>,
}

/// RAM inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemoryScan {
    /// Total visible memory reported by Windows.
    pub total_visible_memory_bytes: Option<u64>,
    /// Free physical memory reported by Windows.
    pub free_physical_memory_bytes: Option<u64>,
    /// Physical memory modules discovered by WMI.
    pub modules: Vec<MemoryModuleScanItem>,
}

/// Physical memory module inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemoryModuleScanItem {
    /// Module manufacturer.
    pub manufacturer: Option<String>,
    /// Module part number.
    pub part_number: Option<String>,
    /// Module capacity in bytes.
    pub capacity_bytes: Option<u64>,
    /// Module speed in MHz.
    pub speed_mhz: Option<u32>,
}

/// CPU inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CpuScanItem {
    /// Processor name.
    pub name: String,
    /// Processor manufacturer.
    pub manufacturer: Option<String>,
    /// WMI architecture code.
    pub architecture: Option<String>,
    /// Physical core count.
    pub physical_cores: Option<u32>,
    /// Logical processor count.
    pub logical_processors: Option<u32>,
    /// Maximum clock in MHz.
    pub max_clock_mhz: Option<u32>,
}

/// GPU inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GpuScanItem {
    /// Adapter name.
    pub name: String,
    /// Driver version when available.
    pub driver_version: Option<String>,
    /// Video processor description when available.
    pub video_processor: Option<String>,
    /// Adapter RAM in bytes when available.
    pub adapter_ram_bytes: Option<u64>,
    /// PNP device identifier.
    pub pnp_device_id: Option<String>,
}

/// Storage inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageScan {
    /// Mounted fixed volumes.
    pub volumes: Vec<StorageVolumeScanItem>,
    /// Physical disk inventory.
    pub physical_disks: Vec<PhysicalDiskScanItem>,
    /// Read-only cleanup preview.
    #[serde(default)]
    pub cleanup: StorageCleanupScan,
    /// Storage Sense state.
    #[serde(default)]
    pub storage_sense: StorageSenseScan,
    /// TRIM and Optimize-Volume state.
    #[serde(default)]
    pub trim: StorageTrimScan,
    /// DirectStorage readiness state.
    #[serde(default)]
    pub direct_storage: DirectStorageScan,
}

/// Mounted fixed volume inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageVolumeScanItem {
    /// Drive letter/device ID.
    pub device_id: String,
    /// Volume label.
    pub volume_name: Option<String>,
    /// File system name.
    pub file_system: Option<String>,
    /// Total volume bytes.
    pub total_bytes: Option<u64>,
    /// Free volume bytes.
    pub free_bytes: Option<u64>,
    /// WMI drive type.
    pub drive_type: Option<u32>,
}

/// Physical disk inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhysicalDiskScanItem {
    /// Friendly disk name.
    pub friendly_name: String,
    /// Media type as reported by Windows.
    pub media_type: Option<String>,
    /// Bus type as reported by Windows.
    pub bus_type: Option<String>,
    /// Health status as reported by Windows.
    pub health_status: Option<String>,
    /// Disk size in bytes.
    pub size_bytes: Option<u64>,
}

/// Read-only storage cleanup preview.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageCleanupScan {
    /// Candidate temp/cache locations and size estimates.
    pub candidates: Vec<StorageCleanupCandidateScanItem>,
    /// Exclusions that must remain outside cleanup.
    pub excluded_patterns: Vec<String>,
}

/// One cleanup preview candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageCleanupCandidateScanItem {
    /// Logical candidate target.
    pub target: String,
    /// Candidate path or location label.
    pub path: String,
    /// Candidate kind.
    pub kind: String,
    /// Estimated reclaimable bytes.
    pub reclaimable_bytes: u64,
    /// Estimated file count.
    pub file_count: u32,
    /// Whether this candidate is safe to show in cleanup preview.
    pub safe_to_preview: bool,
}

/// Storage Sense registry state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageSenseScan {
    /// Whether Storage Sense appears enabled.
    pub enabled: Option<bool>,
    /// Configured cleanup cadence in days.
    pub cadence_days: Option<u32>,
    /// Configured recycle-bin cleanup age in days.
    pub recycle_bin_cleanup_days: Option<u32>,
    /// Configured downloads cleanup age in days.
    pub downloads_cleanup_days: Option<u32>,
    /// Read-only source used for the scan.
    pub source: String,
}

/// TRIM and Optimize-Volume state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageTrimScan {
    /// NTFS DisableDeleteNotify value.
    pub ntfs_disable_delete_notify: Option<u32>,
    /// ReFS DisableDeleteNotify value.
    pub refs_disable_delete_notify: Option<u32>,
    /// Whether Optimize-Volume is available.
    pub optimize_volume_available: bool,
    /// Read-only source used for the scan.
    pub source: String,
}

/// DirectStorage readiness state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DirectStorageScan {
    /// Whether the OS build supports DirectStorage.
    pub os_supported: Option<bool>,
    /// Whether an NVMe disk is present.
    pub nvme_present: Option<bool>,
    /// Whether GPU decompression support is known.
    pub gpu_decompression_supported: Option<bool>,
    /// Bus type of the game volume when known.
    pub game_volume_bus_type: Option<String>,
    /// Read-only source used for the scan.
    pub source: String,
}

/// Network adapter inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkAdapterScanItem {
    /// Adapter name.
    pub name: String,
    /// Adapter type.
    pub adapter_type: Option<String>,
    /// MAC address.
    pub mac_address: Option<String>,
    /// Windows connection label.
    pub net_connection_id: Option<String>,
    /// WMI connection status code.
    pub net_connection_status: Option<u32>,
    /// Link speed in bits per second.
    pub speed_bits_per_second: Option<u64>,
    /// Adapter power-management values exposed by Windows.
    #[serde(default)]
    pub power_management: NetworkAdapterPowerManagementScan,
    /// Advanced adapter properties exposed by the driver.
    #[serde(default)]
    pub advanced_properties: Vec<NetworkAdapterAdvancedPropertyScanItem>,
}

/// Network adapter power-management state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkAdapterPowerManagementScan {
    /// Whether Windows may power down the adapter to save energy.
    pub allow_computer_to_turn_off_device: Option<bool>,
    /// Read-only source used for this adapter power-management scan.
    pub source: String,
}

impl Default for NetworkAdapterPowerManagementScan {
    fn default() -> Self {
        Self {
            allow_computer_to_turn_off_device: None,
            source: "unavailable".to_owned(),
        }
    }
}

/// Advanced network adapter property exposed by `Get-NetAdapterAdvancedProperty`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkAdapterAdvancedPropertyScanItem {
    /// Exact display name exposed by the adapter driver.
    pub display_name: String,
    /// Current display value.
    pub display_value: Option<String>,
    /// Adapter registry keyword for the exact property.
    pub registry_keyword: Option<String>,
    /// Current registry value when exposed by the cmdlet.
    pub registry_value: Option<String>,
}

/// Service inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceScanItem {
    /// Service name.
    pub name: String,
    /// Display name.
    pub display_name: Option<String>,
    /// Current service state.
    pub state: Option<String>,
    /// Service start mode.
    pub start_mode: Option<String>,
}

/// Scheduled task inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScheduledTaskScanItem {
    /// Task name.
    pub task_name: String,
    /// Task path.
    pub task_path: String,
    /// Task state.
    pub state: Option<String>,
    /// Next run time in read-only scan output, when Windows exposes it.
    #[serde(default)]
    pub next_run_time: Option<String>,
    /// Last run time in read-only scan output, when Windows exposes it.
    #[serde(default)]
    pub last_run_time: Option<String>,
}

/// Startup app inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartupAppScanItem {
    /// Startup entry name.
    pub name: String,
    /// Startup command.
    pub command: Option<String>,
    /// Startup entry location.
    pub location: Option<String>,
    /// Owning user when reported by Windows.
    pub user: Option<String>,
    /// Whether Windows reports the entry as enabled.
    pub enabled: Option<bool>,
    /// Startup impact text when Windows exposes it.
    pub startup_impact: Option<String>,
}

/// Background app permission/activity inventory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackgroundAppScanItem {
    /// Package, app, or registry identifier.
    pub app_id: String,
    /// Friendly display name when exposed.
    pub display_name: Option<String>,
    /// Whether background permission appears enabled.
    pub enabled: Option<bool>,
    /// Raw Disabled value from Windows background app settings.
    pub disabled: Option<bool>,
    /// Raw DisabledByUser value from Windows background app settings.
    pub disabled_by_user: Option<bool>,
    /// Activity/impact text when supplied by fixtures or future collectors.
    pub activity: Option<String>,
}

/// Active power plan inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PowerPlanScan {
    /// Active scheme GUID parsed from powercfg output.
    pub active_scheme_guid: Option<String>,
    /// Active scheme name parsed from powercfg output.
    pub active_scheme_name: Option<String>,
    /// Read-only source command.
    pub source: String,
}

/// Read-only scheduler registry state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SchedulerRegistryScan {
    /// MMCSS `SystemResponsiveness` DWORD.
    pub mmcss_system_responsiveness: Option<u32>,
    /// `Win32PrioritySeparation` DWORD.
    pub win32_priority_separation: Option<u32>,
    /// Read-only source used for this scheduler scan.
    pub source: String,
}

impl Default for SchedulerRegistryScan {
    fn default() -> Self {
        Self {
            mmcss_system_responsiveness: None,
            win32_priority_separation: None,
            source: "unavailable".to_owned(),
        }
    }
}

/// Read-only Windows Graphics settings state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GraphicsSettingsScan {
    /// Hardware accelerated GPU scheduling state.
    pub hags: GraphicsDwordSettingScan,
    /// Optimizations for windowed games state.
    pub windowed_optimizations: GraphicsDwordSettingScan,
    /// Variable refresh rate state.
    pub variable_refresh_rate: GraphicsDwordSettingScan,
    /// Whether Windows exposes a high-performance GPU app preference choice.
    pub high_performance_gpu_available: Option<bool>,
    /// Per-app Windows Graphics preference entries.
    #[serde(default)]
    pub app_preferences: Vec<GraphicsAppPreferenceScanItem>,
    /// Read-only source used for this scan section.
    pub source: String,
}

impl Default for GraphicsSettingsScan {
    fn default() -> Self {
        Self {
            hags: GraphicsDwordSettingScan {
                source: "HKLM GraphicsDrivers HwSchMode".to_owned(),
                ..GraphicsDwordSettingScan::default()
            },
            windowed_optimizations: GraphicsDwordSettingScan {
                source: "HKCU DirectX GraphicsSettings SwapEffectUpgradeEnable".to_owned(),
                ..GraphicsDwordSettingScan::default()
            },
            variable_refresh_rate: GraphicsDwordSettingScan {
                source: "HKCU DirectX GraphicsSettings VRR".to_owned(),
                ..GraphicsDwordSettingScan::default()
            },
            high_performance_gpu_available: None,
            app_preferences: Vec::new(),
            source: "unavailable".to_owned(),
        }
    }
}

/// One DWORD-like Windows Graphics settings value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GraphicsDwordSettingScan {
    /// Current DWORD value when available.
    pub value: Option<u32>,
    /// Whether the setting appears available for this machine.
    pub supported: Option<bool>,
    /// Read-only source used for this value.
    pub source: String,
}

/// One per-app Windows Graphics preference registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GraphicsAppPreferenceScanItem {
    /// Executable path stored by Windows Graphics settings.
    pub executable_path: String,
    /// Raw preference payload, such as `GpuPreference=2;`.
    pub preference: String,
}

/// Security state inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SecurityScan {
    /// Device Guard/VBS state.
    pub device_guard: DeviceGuardScan,
    /// HVCI registry state.
    pub hvci: HvciScan,
    /// Windows optional features relevant to virtualization tradeoffs.
    #[serde(default)]
    pub optional_features: Vec<WindowsOptionalFeatureScanItem>,
    /// Defender read-only state.
    pub defender: DefenderScan,
}

/// Device Guard/VBS state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeviceGuardScan {
    /// VBS status code from Win32_DeviceGuard.
    pub virtualization_based_security_status: Option<u32>,
    /// Security services configured codes.
    pub security_services_configured: Vec<u32>,
    /// Security services running codes.
    pub security_services_running: Vec<u32>,
    /// User-mode code integrity policy enforcement status.
    pub usermode_code_integrity_policy_enforcement_status: Option<u32>,
    /// Code integrity policy enforcement status.
    pub code_integrity_policy_enforcement_status: Option<u32>,
}

/// Hypervisor-enforced Code Integrity state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HvciScan {
    /// Enabled registry value when available.
    pub enabled: Option<u32>,
    /// Locked registry value when available.
    pub locked: Option<u32>,
}

/// Windows optional feature state from `Win32_OptionalFeature`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowsOptionalFeatureScanItem {
    /// Feature name, such as `VirtualMachinePlatform`.
    pub name: String,
    /// Windows feature caption when available.
    pub caption: Option<String>,
    /// Install state code from `Win32_OptionalFeature`.
    pub install_state: Option<u32>,
}

/// Microsoft Defender read-only state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DefenderScan {
    /// Whether Defender antivirus is enabled.
    pub antivirus_enabled: Option<bool>,
    /// Whether real-time protection is enabled.
    pub real_time_protection_enabled: Option<bool>,
    /// Whether Tamper Protection is enabled.
    pub tamper_protected: Option<bool>,
    /// Whether antispyware protection is enabled.
    pub antispyware_enabled: Option<bool>,
    /// Existing Defender exclusion paths, read-only.
    #[serde(default)]
    pub exclusion_paths: Vec<String>,
    /// Defender scheduled scan day preference, when available.
    #[serde(default)]
    pub scan_schedule_day: Option<u32>,
    /// Defender scheduled scan time preference, when available.
    #[serde(default)]
    pub scan_schedule_time: Option<String>,
}

/// Reboot-required registry marker state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RebootRequiredScan {
    /// Component Based Servicing reboot marker.
    pub component_based_servicing: bool,
    /// Windows Update reboot marker.
    pub windows_update: bool,
    /// Pending file rename marker.
    pub pending_file_rename: bool,
}

impl RebootRequiredScan {
    /// Returns true when any reboot-required marker is present.
    #[must_use]
    pub const fn is_reboot_required(&self) -> bool {
        self.component_based_servicing || self.windows_update || self.pending_file_rename
    }
}

/// Non-fatal collection error for one scan section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScanCollectionError {
    /// Section identifier.
    pub section: String,
    /// Error message.
    pub message: String,
}

/// Error raised by the system scan adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemScanError {
    reason: SystemScanErrorReason,
    message: String,
}

impl SystemScanError {
    #[cfg(not(windows))]
    fn unsupported_platform() -> Self {
        Self {
            reason: SystemScanErrorReason::UnsupportedPlatform,
            message: "read-only system scan is only available on Windows".to_owned(),
        }
    }

    #[cfg(windows)]
    fn command_failed(message: impl Into<String>) -> Self {
        Self {
            reason: SystemScanErrorReason::CommandFailed,
            message: message.into(),
        }
    }

    fn parse(message: impl Into<String>) -> Self {
        Self {
            reason: SystemScanErrorReason::ParseFailed,
            message: message.into(),
        }
    }

    /// Returns the stable error reason.
    #[must_use]
    pub const fn reason(&self) -> SystemScanErrorReason {
        self.reason
    }

    /// Returns the error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Stable system scan error reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemScanErrorReason {
    /// Live scans are only supported on Windows.
    UnsupportedPlatform,
    /// The static read-only collection command failed.
    CommandFailed,
    /// The scanner emitted malformed JSON.
    ParseFailed,
}

impl SystemScanErrorReason {
    /// Returns a stable string representation for IPC error DTOs.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::CommandFailed => "command_failed",
            Self::ParseFailed => "parse_failed",
        }
    }
}

impl fmt::Display for SystemScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.reason.as_str(), self.message)
    }
}

impl std::error::Error for SystemScanError {}

#[cfg(windows)]
fn run_live_scan() -> Result<SystemScanReport, SystemScanError> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            LIVE_SCAN_SCRIPT,
        ])
        .output()
        .map_err(|source| SystemScanError::command_failed(source.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(SystemScanError::command_failed(if stderr.is_empty() {
            format!("PowerShell scan exited with status {}", output.status)
        } else {
            stderr
        }));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if stdout.is_empty() {
        return Err(SystemScanError::command_failed(
            "PowerShell scan emitted no JSON payload",
        ));
    }

    parse_system_scan_report(&stdout)
}

#[cfg(not(windows))]
fn run_live_scan() -> Result<SystemScanReport, SystemScanError> {
    Err(SystemScanError::unsupported_platform())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../tests/fixtures/system_scan.json");

    #[test]
    fn parses_fixture_report() {
        let report = parse_system_scan_report(FIXTURE).expect("fixture should parse");

        assert!(report.covers_t040_inventory());
        assert_eq!(report.security.hvci.enabled, Some(1));
        assert_eq!(report.scheduler.mmcss_system_responsiveness, Some(20));
        assert_eq!(report.scheduler.win32_priority_separation, Some(2));
        assert!(!report.reboot_required.is_reboot_required());
        assert!(report.collection_errors.is_empty());
    }

    #[test]
    fn rejects_malformed_scan_json() {
        let error = parse_system_scan_report("{").expect_err("malformed JSON must fail");

        assert_eq!(error.reason(), SystemScanErrorReason::ParseFailed);
    }
}
