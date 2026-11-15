param(
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [ValidateRange(1, 30)][int]$Minutes = 20
)
$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'Use the existing PowerShell 7 host and its configured policy; do not change system policy.'
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'ETW requires an elevated Administrator token. No settings have been changed.'
}
$captureRoot = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $captureRoot) { throw 'Choose a new output directory.' }
New-Item -ItemType Directory -Path $captureRoot | Out-Null
$sessionName = 'KRAQEL-M3-' + [guid]::NewGuid().ToString('N')
$traceFile = Join-Path $captureRoot 'network.etl'
$providerFile = Join-Path $captureRoot 'providers.txt'
@(
    '{7DD42A49-5329-4832-8DFD-43D979153A88} 0x30 4',
    '{22FB2CD6-0E7B-422B-A0C7-2FAD1FD0E716} 0x10 4'
) | Set-Content -LiteralPath $providerFile -Encoding Ascii
$started = $false
$controlPorts = @()
try {
    $log = & logman.exe create trace $sessionName -pf $providerFile -o $traceFile -f bin -max 128 -bs 64 -nb 16 64 -ets 2>&1
    if ($LASTEXITCODE -ne 0) { throw ($log -join [Environment]::NewLine) }
    $started = $true
    # Positive controls are localhost only, from this helper PID, never from KRAQEL.
    foreach ($hostAddress in @('127.0.0.1', '::1')) {
        $address = [Net.IPAddress]::Parse($hostAddress)
        $listener = New-Object Net.Sockets.TcpListener($address, 0)
        $client = New-Object Net.Sockets.TcpClient($address.AddressFamily)
        $accepted = $null
        try {
            $listener.Start()
            $port = $listener.LocalEndpoint.Port
            $client.Connect($address, $port)
            $accepted = $listener.AcceptTcpClient()
            $client.GetStream().WriteByte(42)
            if ($accepted.GetStream().ReadByte() -ne 42) { throw 'TCP control failed.' }
            $controlPorts += @{ address = $hostAddress; port = $port; protocol = 'TCP' }
        } finally {
            if ($accepted) { $accepted.Dispose() }
            $client.Dispose()
            $listener.Stop()
        }
        $receiver = New-Object Net.Sockets.UdpClient($address.AddressFamily)
        $sender = New-Object Net.Sockets.UdpClient($address.AddressFamily)
        try {
            $receiver.Client.Bind((New-Object Net.IPEndPoint($address, 0)))
            $receiver.Client.ReceiveTimeout = 5000
            $port = $receiver.Client.LocalEndPoint.Port
            $destination = New-Object Net.IPEndPoint($address, $port)
            [void]$sender.Send([byte[]]@(42), 1, $destination)
            $remote = New-Object Net.IPEndPoint($address, 0)
            $bytes = $receiver.Receive([ref]$remote)
            if ($bytes.Length -ne 1 -or $bytes[0] -ne 42) { throw 'UDP control failed.' }
            $controlPorts += @{ address = $hostAddress; port = $port; protocol = 'UDP' }
        } finally { $receiver.Dispose(); $sender.Dispose() }
    }
    $ready = @{
        startedUtc = [DateTime]::UtcNow.ToString('o'); helperPid = $PID
        session = $sessionName; controls = $controlPorts; maxMinutes = $Minutes
        stopFile = (Join-Path $captureRoot 'STOP')
    }
    $ready | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $captureRoot 'ready.json') -Encoding UTF8
    $deadline = [DateTime]::UtcNow.AddMinutes($Minutes)
    while ([DateTime]::UtcNow -lt $deadline -and -not (Test-Path -LiteralPath $ready.stopFile)) {
        Start-Sleep -Milliseconds 500
    }
} catch {
    @{ error = $_.Exception.Message } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $captureRoot 'error.json') -Encoding UTF8
    throw
} finally {
    if ($started) {
        $stopLog = & logman.exe stop $sessionName -ets 2>&1
        $traceStoppedUtc = [DateTime]::UtcNow.ToString('o')
        $stopLog | Set-Content -LiteralPath (Join-Path $captureRoot 'stop.log') -Encoding UTF8
        if ($LASTEXITCODE -ne 0) { throw 'Could not stop the owned trace session.' }
    }
}
$conversion = & tracerpt.exe $traceFile -o (Join-Path $captureRoot 'events.xml') -of XML -summary (Join-Path $captureRoot 'summary.xml') -y 2>&1
$conversion | Set-Content -LiteralPath (Join-Path $captureRoot 'conversion.log') -Encoding UTF8
if ($LASTEXITCODE -ne 0) { throw 'tracerpt conversion failed; retain raw ETL locally.' }
@{ traceStoppedUtc = $traceStoppedUtc; convertedUtc = [DateTime]::UtcNow.ToString('o') } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $captureRoot 'complete.json') -Encoding UTF8
