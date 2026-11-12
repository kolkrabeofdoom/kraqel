[CmdletBinding()]
param(
    [switch]$AllowDirtyWorkingTree,
    [switch]$Overwrite
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $FilePath $($Arguments -join ' ')"
    }
}

function Read-CommandVersion {
    param(
        [Parameter(Mandatory = $true)]
        [string]$FilePath,
        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    $output = & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Could not read tool version: $FilePath $($Arguments -join ' ')"
    }

    return (($output | Out-String).Trim())
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

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'The KRAQEL Windows release must be built on Windows.'
}

$repositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$previousLocation = Get-Location

try {
    Set-Location -LiteralPath $repositoryRoot

    $gitRoot = (git rev-parse --show-toplevel).Trim()
    if ($LASTEXITCODE -ne 0 -or [IO.Path]::GetFullPath($gitRoot) -ne $repositoryRoot) {
        throw "The script must run from the KRAQEL repository at $repositoryRoot."
    }

    $workingTreeChanges = @(git status --porcelain --untracked-files=normal)
    if ($LASTEXITCODE -ne 0) {
        throw 'Could not inspect the Git working tree.'
    }
    $isDirty = $workingTreeChanges.Count -gt 0
    if ($isDirty -and -not $AllowDirtyWorkingTree) {
        throw 'The Git working tree is not clean. Commit the release sources or use -AllowDirtyWorkingTree for a non-publishable development build.'
    }

    $baseConfigPath = Join-Path $repositoryRoot 'apps\desktop\src-tauri\tauri.conf.json'
    $baseConfig = Get-Content -LiteralPath $baseConfigPath -Raw | ConvertFrom-Json
    $rootPackage = Get-Content -LiteralPath (Join-Path $repositoryRoot 'package.json') -Raw | ConvertFrom-Json
    $version = [string]$baseConfig.version
    if ([string]::IsNullOrWhiteSpace($version) -or $version -ne [string]$rootPackage.version) {
        throw 'The versions in tauri.conf.json and package.json must be identical and non-empty.'
    }

    $commit = (git rev-parse HEAD).Trim()
    $shortCommit = (git rev-parse --short=12 HEAD).Trim()
    $branch = (git branch --show-current).Trim()
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($commit)) {
        throw 'Could not determine the source commit.'
    }

    $targetTriple = 'x86_64-pc-windows-msvc'
    Write-Host "Building KRAQEL $version for $targetTriple from $shortCommit ..."
    Invoke-Checked -FilePath 'pnpm' -Arguments @(
        '--filter',
        '@kraqel/desktop',
        'exec',
        'tauri',
        'build',
        '--bundles',
        'nsis',
        '--target',
        $targetTriple,
        '--config',
        'src-tauri/tauri.release.conf.json'
    )

    $bundleDirectory = Join-Path $repositoryRoot "target\$targetTriple\release\bundle\nsis"
    $expectedSourceName = "KRAQEL_${version}_x64-setup.exe"
    $sourceInstaller = Join-Path $bundleDirectory $expectedSourceName
    $allInstallers = @(Get-ChildItem -LiteralPath $bundleDirectory -Filter '*-setup.exe' -File)
    if ($allInstallers.Count -ne 1 -or -not (Test-Path -LiteralPath $sourceInstaller -PathType Leaf)) {
        $found = ($allInstallers.Name -join ', ')
        throw "Expected exactly $expectedSourceName in $bundleDirectory, found: $found"
    }

    if (Test-EmbeddedAuthenticodeSignature -Path $sourceInstaller) {
        throw 'Release policy requires an unsigned artifact, but the setup contains an Authenticode certificate table.'
    }

    $outputDirectory = Join-Path $repositoryRoot "artifacts\release\$version\windows-x64\$shortCommit"
    $artifactName = "KRAQEL_${version}_windows-x64-setup.exe"
    $checksumName = "KRAQEL_${version}_windows-x64-SHA256SUMS.txt"
    $manifestName = "KRAQEL_${version}_windows-x64-build.json"
    $artifactPath = Join-Path $outputDirectory $artifactName
    $checksumPath = Join-Path $outputDirectory $checksumName
    $manifestPath = Join-Path $outputDirectory $manifestName
    $outputFiles = @($artifactPath, $checksumPath, $manifestPath)
    $existingFiles = @($outputFiles | Where-Object { Test-Path -LiteralPath $_ })
    if ($existingFiles.Count -gt 0 -and -not $Overwrite) {
        throw "Release output already exists. Review it or rerun with -Overwrite: $outputDirectory"
    }

    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
    Copy-Item -LiteralPath $sourceInstaller -Destination $artifactPath -Force:$Overwrite
    $artifact = Get-Item -LiteralPath $artifactPath
    $sha256 = (Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
    Set-Content -LiteralPath $checksumPath -Value "$sha256  $artifactName" -Encoding ascii

    $manifest = [ordered]@{
        schemaVersion = 1
        product = 'KRAQEL'
        version = $version
        status = if ($isDirty) { 'development-only-dirty' } else { 'unsigned-release-candidate' }
        createdAtUtc = [DateTime]::UtcNow.ToString('o')
        artifact = [ordered]@{
            file = $artifactName
            sizeBytes = $artifact.Length
            sha256 = $sha256
            authenticodeStatus = 'NotSigned'
        }
        source = [ordered]@{
            commit = $commit
            shortCommit = $shortCommit
            branch = $branch
            dirty = $isDirty
        }
        build = [ordered]@{
            target = $targetTriple
            bundle = 'nsis'
            configuration = 'apps/desktop/src-tauri/tauri.release.conf.json'
            node = Read-CommandVersion -FilePath 'node' -Arguments @('--version')
            pnpm = Read-CommandVersion -FilePath 'pnpm' -Arguments @('--version')
            rustc = Read-CommandVersion -FilePath 'rustc' -Arguments @('--version')
            cargo = Read-CommandVersion -FilePath 'cargo' -Arguments @('--version')
            tauri = Read-CommandVersion -FilePath 'pnpm' -Arguments @('--filter', '@kraqel/desktop', 'exec', 'tauri', '--version')
        }
        distribution = [ordered]@{
            signed = $false
            installMode = 'currentUser'
            webviewInstallMode = 'offlineInstaller'
            updaterArtifacts = $false
        }
    }
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding utf8

    $verifiedHash = (Get-FileHash -LiteralPath $artifactPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($verifiedHash -ne $sha256) {
        throw 'The staged artifact changed while its release metadata was being written.'
    }

    Write-Host "Release candidate staged at $outputDirectory"
    Write-Host "SHA-256: $sha256"
    if ($isDirty) {
        Write-Warning 'This build used an uncommitted working tree and must not be published.'
    }
}
finally {
    Set-Location -LiteralPath $previousLocation
}
