[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ArtifactDirectory,
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9a-fA-F]{40}$')]
    [string]$ExpectedCommit
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$version = '0.1.0'
$artifactName = "KRAQEL_${version}_windows-x64-setup.exe"
$checksumName = "KRAQEL_${version}_windows-x64-SHA256SUMS.txt"
$manifestName = "KRAQEL_${version}_windows-x64-build.json"
$installRoot = Join-Path ([IO.Path]::GetTempPath()) 'kraqel-release-acceptance'
$appDataRoot = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'org.kraqel.desktop'
$uninstallRegistryPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*'

function Get-Sha256Hex {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [IO.FileShare]$Share = [IO.FileShare]::Read
    )

    $algorithm = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, $Share)
    try {
        $hash = $algorithm.ComputeHash($stream)
        return [BitConverter]::ToString($hash).Replace('-', '').ToLowerInvariant()
    }
    finally {
        $stream.Dispose()
        $algorithm.Dispose()
    }
}

function Get-TextSha256Hex {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Text
    )

    $algorithm = [Security.Cryptography.SHA256]::Create()
    try {
        $hash = $algorithm.ComputeHash([Text.Encoding]::UTF8.GetBytes($Text))
        return [BitConverter]::ToString($hash).Replace('-', '').ToLowerInvariant()
    }
    finally {
        $algorithm.Dispose()
    }
}

function Test-EmbeddedAuthenticodeSignature {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    $reader = [IO.BinaryReader]::new($stream)
    try {
        if ($reader.ReadUInt16() -ne 0x5a4d) {
            throw "Expected a Windows PE file, but the DOS header is invalid: $Path"
        }

        $stream.Position = 0x3c
        $peHeaderOffset = $reader.ReadInt32()
        if ($peHeaderOffset -lt 0 -or $peHeaderOffset -gt ($stream.Length - 32)) {
            throw "The PE header offset is outside the file: $Path"
        }

        $stream.Position = $peHeaderOffset
        if ($reader.ReadUInt32() -ne 0x00004550) {
            throw "The Windows PE signature is invalid: $Path"
        }

        $stream.Position = $peHeaderOffset + 20
        $optionalHeaderSize = $reader.ReadUInt16()
        $optionalHeaderOffset = $peHeaderOffset + 24
        $stream.Position = $optionalHeaderOffset
        $optionalHeaderMagic = $reader.ReadUInt16()
        $dataDirectoryOffset = switch ($optionalHeaderMagic) {
            0x010b { 96 }
            0x020b { 112 }
            default { throw "Unsupported PE optional-header format: 0x$($optionalHeaderMagic.ToString('x4'))" }
        }

        $securityDirectoryOffset = $dataDirectoryOffset + (4 * 8)
        if ($optionalHeaderSize -lt ($securityDirectoryOffset + 8)) {
            throw "The PE optional header does not contain a security directory: $Path"
        }

        $stream.Position = $optionalHeaderOffset + $securityDirectoryOffset
        $certificateTableOffset = $reader.ReadUInt32()
        $certificateTableSize = $reader.ReadUInt32()
        return $certificateTableOffset -ne 0 -or $certificateTableSize -ne 0
    }
    finally {
        $reader.Dispose()
        $stream.Dispose()
    }
}

function Get-AppDataFingerprint {
    if (-not (Test-Path -LiteralPath $appDataRoot -PathType Container)) {
        return Get-TextSha256Hex -Text 'ABSENT'
    }

    $rootWithSeparator = [IO.Path]::GetFullPath($appDataRoot).TrimEnd('\') + '\'
    $records = @(
        Get-ChildItem -LiteralPath $appDataRoot -Force -Recurse | Sort-Object FullName | ForEach-Object {
            $relativePath = $_.FullName.Substring($rootWithSeparator.Length).Replace('\', '/')
            if ($_.PSIsContainer) {
                "D|$relativePath"
            }
            else {
                $fileHash = Get-Sha256Hex -Path $_.FullName -Share ([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete)
                "F|$relativePath|$($_.Length)|$($_.LastWriteTimeUtc.Ticks)|$fileHash"
            }
        }
    )
    return Get-TextSha256Hex -Text ($records -join "`n")
}

function Get-KraqelUninstallEntries {
    @(
        Get-ItemProperty $uninstallRegistryPath -ErrorAction SilentlyContinue |
            Where-Object { $_.DisplayName -eq 'KRAQEL' }
    )
}

function Remove-AcceptanceDirectory {
    $resolvedTemp = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
    $resolvedTarget = [IO.Path]::GetFullPath($installRoot).TrimEnd('\')
    if (-not $resolvedTarget.Equals(
            "$resolvedTemp\kraqel-release-acceptance",
            [StringComparison]::OrdinalIgnoreCase
        )) {
        throw "Unsafe cleanup target rejected: $resolvedTarget"
    }
    Remove-Item -LiteralPath $resolvedTarget -Recurse -Force -ErrorAction SilentlyContinue
}

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'The KRAQEL release artifact smoke test must run on Windows.'
}

$resolvedArtifactDirectory = (Resolve-Path -LiteralPath $ArtifactDirectory).Path
$expectedNames = @($artifactName, $checksumName, $manifestName) | Sort-Object
$actualFiles = @(Get-ChildItem -LiteralPath $resolvedArtifactDirectory -File)
$actualNames = @($actualFiles.Name | Sort-Object)
if ($actualFiles.Count -ne 3 -or ($actualNames -join "`n") -ne ($expectedNames -join "`n")) {
    throw "Expected exactly the setup, checksum and build manifest in $resolvedArtifactDirectory."
}

$artifactPath = Join-Path $resolvedArtifactDirectory $artifactName
$checksumPath = Join-Path $resolvedArtifactDirectory $checksumName
$manifestPath = Join-Path $resolvedArtifactDirectory $manifestName
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$checksumLine = Get-Content -LiteralPath $checksumPath -Raw
if ($checksumLine -notmatch "^([0-9a-f]{64})  $([Regex]::Escape($artifactName))\s*$") {
    throw 'The checksum file has an unexpected format or filename.'
}

$checksumHash = $Matches[1]
$artifactHash = Get-Sha256Hex -Path $artifactPath
if ($artifactHash -ne $checksumHash -or $artifactHash -ne [string]$manifest.artifact.sha256) {
    throw 'The setup, checksum file and build manifest do not have the same SHA-256.'
}
if ([string]$manifest.version -ne $version -or [string]$manifest.product -ne 'KRAQEL') {
    throw 'The build manifest identifies an unexpected product or version.'
}
if ([string]$manifest.source.commit -ne $ExpectedCommit.ToLowerInvariant() -or $manifest.source.dirty) {
    throw 'The build manifest does not identify the expected clean source commit.'
}
if ([string]$manifest.status -ne 'unsigned-release-candidate' -or
    [string]$manifest.artifact.authenticodeStatus -ne 'NotSigned') {
    throw 'The build manifest does not identify an unsigned release candidate.'
}
if ([string]$manifest.distribution.installMode -ne 'currentUser' -or
    [string]$manifest.distribution.webviewInstallMode -ne 'offlineInstaller' -or
    $manifest.distribution.updaterArtifacts) {
    throw 'The build manifest contains unexpected distribution settings.'
}
if (Test-EmbeddedAuthenticodeSignature -Path $artifactPath) {
    throw 'The downloaded setup contains an Authenticode certificate table.'
}

if (@(Get-KraqelUninstallEntries).Count -ne 0) {
    throw 'A regular KRAQEL installation is already registered. The acceptance test will not replace it.'
}
if (Test-Path -LiteralPath $installRoot) {
    throw "The isolated install target already exists and will not be overwritten: $installRoot"
}

$appDataFingerprintBefore = Get-AppDataFingerprint
$installed = $false
try {
    $installerProcess = Start-Process -FilePath $artifactPath -ArgumentList @('/S', "/D=$installRoot") -Wait -PassThru
    if ($installerProcess.ExitCode -ne 0) {
        throw "The NSIS installer exited with code $($installerProcess.ExitCode)."
    }
    $installed = $true

    $entriesAfterInstall = @(Get-KraqelUninstallEntries)
    if ($entriesAfterInstall.Count -ne 1) {
        throw "Expected exactly one KRAQEL uninstall entry, found: $($entriesAfterInstall.Count)."
    }
    if ([string]$entriesAfterInstall[0].DisplayVersion -ne $version) {
        throw "Unexpected registered version: $($entriesAfterInstall[0].DisplayVersion)"
    }
    $registeredLocation = [IO.Path]::GetFullPath(([string]$entriesAfterInstall[0].InstallLocation).Trim('"')).TrimEnd('\')
    if (-not $registeredLocation.Equals($installRoot.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) {
        throw "Unexpected registered install location: $registeredLocation"
    }

    $applicationCandidates = @(
        Get-ChildItem -LiteralPath $installRoot -Filter '*.exe' -File |
            Where-Object { $_.Name -ne 'uninstall.exe' }
    )
    if ($applicationCandidates.Count -ne 1) {
        throw "Expected exactly one installed application executable, found: $($applicationCandidates.Count)."
    }

    $uninstaller = Join-Path $installRoot 'uninstall.exe'
    if (-not (Test-Path -LiteralPath $uninstaller -PathType Leaf)) {
        throw 'The expected NSIS uninstaller is missing.'
    }
    $uninstallerProcess = Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait -PassThru
    if ($uninstallerProcess.ExitCode -ne 0) {
        throw "The NSIS uninstaller exited with code $($uninstallerProcess.ExitCode)."
    }
    $installed = $false

    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while (@(Get-KraqelUninstallEntries).Count -ne 0 -and [DateTime]::UtcNow -lt $deadline) {
        [Threading.Thread]::Sleep(100)
    }
    if (@(Get-KraqelUninstallEntries).Count -ne 0) {
        throw 'The KRAQEL uninstall entry was not removed.'
    }
    if (Test-Path -LiteralPath $applicationCandidates[0].FullName) {
        throw 'The application executable remained after uninstallation.'
    }

    $appDataFingerprintAfter = Get-AppDataFingerprint
    if ($appDataFingerprintAfter -ne $appDataFingerprintBefore) {
        throw 'The regular KRAQEL app-data directory changed during installer acceptance.'
    }

    Write-Output 'Windows release artifact smoke test passed.'
    Write-Output "Artifact: $artifactPath"
    Write-Output "SHA-256: $artifactHash"
    Write-Output "Source commit: $($manifest.source.commit)"
    Write-Output 'Evidence: downloaded unsigned artifact, manifest, checksum, isolated installation, registration and uninstallation.'
    Write-Output 'The application was not started and existing KRAQEL app data remained unchanged.'
}
finally {
    if ($installed) {
        $uninstaller = Join-Path $installRoot 'uninstall.exe'
        if (Test-Path -LiteralPath $uninstaller -PathType Leaf) {
            Start-Process -FilePath $uninstaller -ArgumentList '/S' -Wait | Out-Null
        }
    }
    if (Test-Path -LiteralPath $installRoot) {
        Remove-AcceptanceDirectory
    }
}
