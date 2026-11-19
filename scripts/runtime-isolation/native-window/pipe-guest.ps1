param([ValidateSet('read-scene','deny-save','edit-conflict','durable-reopen','commit-crashes','channel-interruptions','window-close','channel-ends','starter-restart','proof-run','proof-incomplete','proof-witness-loss','proof-consumed-loss','proof-live-loss','handshake','reference','exercise')][string]$Trial='commit-crashes')
$ErrorActionPreference='Stop'
$dir='C:\Program Files\KRAQEL-Window-Probe'
try {
    $expected=Get-Content -LiteralPath "$dir\anchor.json" -Raw | ConvertFrom-Json
    if ($env:COMPUTERNAME -ne 'KRAQEL-BP3' -or (Get-CimInstance Win32_ComputerSystemProduct).UUID -ne $expected.uuid -or
        (Get-CimInstance Win32_OperatingSystem).Version -ne $expected.build -or (Get-Process -Id $PID).SessionId -eq 0 -or
        @(Get-NetAdapter -IncludeHidden | Where-Object Status -eq Up).Count -or
        (Get-CimInstance -Namespace root\Microsoft\Windows\DeviceGuard -ClassName Win32_DeviceGuard).UsermodeCodeIntegrityPolicyEnforcementStatus -ne 2) { throw 'Guest boundary mismatch.' }
    $json=& "$env:SystemRoot\System32\CiTool.exe" -lp -json
    if ($LASTEXITCODE -ne 0 -or @((ConvertFrom-Json ($json -join ' ')).Policies | Where-Object { ([string]$_.PolicyID).Trim('{}') -eq $expected.policy -and $_.IsEnforced }).Count -ne 1 -or
        (Get-FileHash "$env:SystemRoot\System32\CodeIntegrity\CiPolicies\Active\{$($expected.policy)}.cip").Hash -ne $expected.policyHash) { throw 'Policy mismatch.' }
    foreach ($file in $expected.files) {
        if ((Get-FileHash -LiteralPath "$dir\$($file.name)").Hash -ne $file.hash) { throw 'Fixture mismatch.' }
    }
    if ($Trial -eq 'reference') { Start-Sleep -Seconds 2; exit 0 }
    if ($Trial -eq 'exercise') { $Trial = 'commit-crashes' }
    if ($Trial -in 'proof-witness-loss','proof-consumed-loss','proof-live-loss') {
        $consumed = $Trial -eq 'proof-consumed-loss'
        $exitCode = if ($Trial -eq 'proof-live-loss') { 0 } elseif ($consumed) { 104 } else { 103 }
        & "$dir\pipe-check.exe" $Trial
        if ($LASTEXITCODE -ne $exitCode) { throw 'Expected witness crash missing.' }
        $path=Join-Path $dir 'pipe.lock'
        $before=(Get-FileHash -LiteralPath $path).Hash
        & "$dir\pipe-check.exe" proof-run
        if ($LASTEXITCODE -ne 102 -or (Get-FileHash -LiteralPath $path).Hash -ne $before) {
            throw 'Fresh witness denial or intact end record missing.'
        }
        Write-Output 'Witness crash confirmed; fresh witness denied; durable state unchanged.'
        exit 0
    }
    if ($Trial -eq 'starter-restart') {
        & "$dir\pipe-check.exe" handshake
        if ($LASTEXITCODE -ne 0) { throw 'Normal completion missing.' }
        & "$dir\pipe-check.exe" starter-crash
        if ($LASTEXITCODE -ne 101) { throw 'Pending transaction crash missing.' }
        # Observe process teardown separately. This is not an end record.
        $until=[DateTime]::UtcNow.AddSeconds(5)
        do {
            $live=@(Get-Process -Name native-leaf,pipe-service,pipe-other -ErrorAction SilentlyContinue)
            if (!$live.Count) { break }
            Start-Sleep -Milliseconds 50
        } while ([DateTime]::UtcNow -lt $until)
        if ($live.Count) { throw 'Probe role still running.' }
        $lock=Join-Path $dir 'pipe.lock'
        $pendingHash=(Get-FileHash -LiteralPath $lock).Hash
        foreach ($attempt in 'reopen-close-read','read-scene','reopen-close-read') {
            & "$dir\pipe-check.exe" $attempt
            if ($LASTEXITCODE -ne 102 -or (Get-FileHash -LiteralPath $lock).Hash -ne $pendingHash) { throw 'Restart denial missing or record changed.' }
        }
        Write-Output 'Crash at pending transaction; roles ended; read, new start and repeated read blocked.'
        exit 0
    }
    & "$dir\pipe-check.exe" $Trial
    exit $LASTEXITCODE
} catch { Write-Output $_.Exception.Message; exit 1 }
