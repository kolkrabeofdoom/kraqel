# Windows-only P3 research fixture. Run with administrative rights for VHD attachment.
# Creates its own source volume. Takes no source, destination or physical disk parameter.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$python = (Get-Command python.exe -CommandType Application | Select-Object -First 1).Source
if (!(Test-Path -LiteralPath (Join-Path $PSScriptRoot 'probe/target/debug/kraqel-isolation-probe.exe') -PathType Leaf)) {
    throw 'Build the local probe first: cargo build --offline --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml --bin kraqel-isolation-probe'
}
$fixture = Join-Path ([IO.Path]::GetTempPath()) ("kraqel-p3-" + [guid]::NewGuid().ToString("N"))
$image = Join-Path $fixture 'source.vhdx'
$attached = $false
$imagePin = $null
function Get-FixtureDisk {
    $disks = @(Get-DiskImage -ImagePath $image | Get-Disk)
    if ($disks.Count -ne 1 -or $disks[0].IsBoot -or $disks[0].IsSystem -or
        $disks[0].Size -ne 128MB -or $disks[0].BusType -ne 'File Backed Virtual') {
        throw 'Unexpected fixture disk identity'
    }
    $disks[0]
}
function Invoke-Fixture([string]$Mode, [string]$VolumePath) {
    & $python -I (Join-Path $PSScriptRoot 'p3-volume-fixture.py') $Mode $VolumePath $fixture
    if ($LASTEXITCODE -ne 0) { throw "P3 fixture failed ($Mode)" }
}
try {
    $null = New-Item -ItemType Directory -Path $fixture
    $null = New-VHD -Path $image -Dynamic -SizeBytes 128MB
    $null = Mount-VHD -Path $image -NoDriveLetter
    $attached = $true
    $disk = Get-FixtureDisk
    if ($disk.PartitionStyle -ne 'RAW' -or $disk.IsReadOnly) { throw 'Expected a fresh writable image' }
    $null = $disk | Initialize-Disk -PartitionStyle GPT
    $partition = Get-FixtureDisk | New-Partition -UseMaximumSize
    $volume = $partition | Format-Volume -FileSystem NTFS -NewFileSystemLabel 'KRAQEL-P3' -Confirm:$false
    $volumePath = $volume.Path
    if ($volumePath -notmatch '^\\\\\?\\Volume\{[0-9a-f-]{36}\}\\$') { throw 'Missing local volume binding' }
    $partitionId = $partition.Guid
    Invoke-Fixture 'prepare' $volumePath

    Dismount-VHD -Path $image
    $attached = $false
    # Bind the detached backing object before allowing the read-only attachment.
    $imagePin = [IO.File]::Open($image, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::Read)
    $null = Mount-VHD -Path $image -ReadOnly -NoDriveLetter
    $attached = $true
    $disk = Get-FixtureDisk
    $reopened = @($disk | Get-Partition | Where-Object Guid -eq $partitionId | Get-Volume)
    if (!$disk.IsReadOnly -or $reopened.Count -ne 1 -or $reopened[0].Path -ne $volumePath) {
        throw 'Read-only source volume not confirmed'
    }
    Invoke-Fixture 'capture' $volumePath
} finally {
    try { if ($attached) { Dismount-VHD -Path $image } } finally { if ($null -ne $imagePin) { $imagePin.Dispose() } }
    # The only removable items are this run's own image and fixed fixture outputs.
    foreach ($name in @('source.vhdx', 'expected.json')) {
        $file = Join-Path $fixture $name
        if ([IO.File]::Exists($file)) { [IO.File]::Delete($file) }
    }
    if ([IO.Directory]::Exists($fixture)) { [IO.Directory]::Delete($fixture) }
}
