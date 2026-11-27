param(
    [Parameter(Mandatory=$true)][string]$Probe,
    [Parameter(Mandatory=$true)][string]$ProbeSha256,
    [Parameter(Mandatory=$true)][string]$UiFixture,
    [Parameter(Mandatory=$true)][string]$UiSha256
)
# Four worker lifecycle boundaries, explicit hashes and fresh synthetic roots only.
# An unbound launch stays blocked; harness cleanup never becomes product authority.
$ErrorActionPreference='Stop'
foreach($pair in @(@($Probe,$ProbeSha256),@($UiFixture,$UiSha256))) {
    if($pair[1] -cnotmatch '^[0-9a-f]{64}$' -or (Get-FileHash -LiteralPath $pair[0]).Hash.ToLowerInvariant() -ne $pair[1]){throw 'Unexpected explicitly approved program hash'}
}
$Probe=(Resolve-Path -LiteralPath $Probe).Path;$UiFixture=(Resolve-Path -LiteralPath $UiFixture).Path
$fixtureRoot=Join-Path $env:TEMP ('kraqel-isolation-controller-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$steps=[Collections.Generic.List[object]]::new();$cases=[Collections.Generic.List[object]]::new();$zero='0'*64
Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
public static class KraqelResourceFixtureCleanup {
    [DllImport("userenv.dll", CharSet=CharSet.Unicode)]
    public static extern int DeleteAppContainerProfile(string name);
}
'@
function Invoke-Step([string]$Action,[string]$Head,[int]$Expected=0,[switch]$Record) {
    $info=[Diagnostics.ProcessStartInfo]::new($Probe)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    $argv=if($Record){@('product-workspace-child',$store,$Action,$UiFixture,$UiSha256,$Head)}else{@('product-workspace-probe-child',$store,$Action,$Head)}
    foreach($arg in $argv){$info.ArgumentList.Add($arg)}
    $process=[Diagnostics.Process]::Start($info)
    try {
        $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
        if(-not $process.WaitForExit(30000)){$process.Kill();$process.WaitForExit();throw 'Resource crash check timed out'}
        $out=$stdout.GetAwaiter().GetResult();$err=$stderr.GetAwaiter().GetResult();$index=$steps.Count
        [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stdout"),$out);[IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stderr"),$err)
        $steps.Add(@{pid=$process.Id;case=$caseName;action=$Action;exitCode=$process.ExitCode;expectedExitCode=$Expected})
        $steps | ConvertTo-Json | Set-Content -LiteralPath "$fixtureRoot/processes.json"
        if($process.ExitCode -ne $Expected){throw 'Unexpected exit; inspect private evidence'}
        if($Expected -eq 0){return $out | ConvertFrom-Json}
    } finally {$process.Dispose()}
}
function Fingerprint {
    $files=@(Get-ChildItem -LiteralPath "$store/control/records" -File | Sort-Object Name)+@(Get-Item -LiteralPath "$store/control/head.mac","$store/control/authority.key")
    return (@($files | ForEach-Object {$_.Name+':'+(Get-FileHash -LiteralPath $_.FullName).Hash}) -join '|')
}
function Read-Own-Execution {
    $domain=[Text.Encoding]::UTF8.GetBytes("KRAQEL-PRODUCT-CONTROL"+[char]0+[char]1)
    $wire=[IO.File]::ReadAllBytes("$store/control/records/0006.mac");$offset=$domain.Length+32
    if($wire.Length -le $offset -or [Convert]::ToHexString($wire[0..($domain.Length-1)]) -ne [Convert]::ToHexString($domain)){throw 'Unexpected record domain'}
    $key=[IO.File]::ReadAllBytes("$store/control/authority.key");$mac=[Security.Cryptography.HMACSHA256]::new($key)
    try {$expected=$mac.ComputeHash([byte[]]($domain+$wire[$offset..($wire.Length-1)]))} finally {$mac.Dispose();[Array]::Clear($key)}
    if(-not [Security.Cryptography.CryptographicOperations]::FixedTimeEquals($expected,[byte[]]$wire[$domain.Length..($offset-1)])){throw 'Invalid fixture MAC'}
    $record=[Text.Encoding]::UTF8.GetString($wire,$offset,$wire.Length-$offset) | ConvertFrom-Json
    $execution=$record.action.execution
    if($record.action.kind -ne 'commitWorkspaceProbe' -or $execution.version -notin @(2,3) -or [Convert]::ToHexString([byte[]]$execution.workerSha256).ToLowerInvariant() -ne $ProbeSha256){throw 'Unexpected own fixture execution'}
    return $execution
}
function Profile-Exists([string]$Moniker) {
    $mappings=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppContainer\Mappings')
    if($null -eq $mappings){throw 'Profile mappings unavailable'}
    try {
        foreach($name in $mappings.GetSubKeyNames()) {
            $entry=$mappings.OpenSubKey($name)
            try {if($entry.GetValue('Moniker') -ceq $Moniker){return $true}} finally {$entry.Dispose()}
        }
        return $false
    } finally {$mappings.Dispose()}
}

function Confirm-Own-Fixture-End($Spawn) {
    $process=$null
    try {
        try {$process=[Diagnostics.Process]::GetProcessById([int]$Spawn.workerIdentity.pid)} catch [ArgumentException] {return}
        if([UInt64]$process.StartTime.ToUniversalTime().ToFileTimeUtc() -eq [UInt64]$Spawn.workerIdentity.creationTime) {
            if(-not $process.WaitForExit(5000)){throw 'Own worker has not ended'}
        }
        # A different creation time proves that the recorded PID generation is gone. No kill.
    } finally {if($null -ne $process){$process.Dispose()}}
}
foreach($point in @('before-bound','after-bound','live','after-exit')) {
    $caseName='case-'+$cases.Count.ToString('D2');$caseRoot=Join-Path $fixtureRoot $caseName
    New-Item -ItemType Directory -Path $caseRoot | Out-Null;$store=Join-Path $caseRoot 'store'
    $state=Invoke-Step init $zero -Record
    $state=Invoke-Step migrate5 $state.control.headSha256 -Record
    $state=Invoke-Step bind $state.control.headSha256 -Record
    $boundHead=$state.control.headSha256;$key=(Get-FileHash -LiteralPath "$store/control/authority.key").Hash
    $sentinel=(Get-FileHash -LiteralPath "$store/data/workspace/sentinel.txt").Hash
    Invoke-Step "crash-$point" $boundHead 86
    $state=Invoke-Step status $zero
    if(-not $state.probePending -or $state.workspace.generation -ne 1){throw 'Missing persisted worker launch'}
    $pendingHead=$state.workspace.control.headSha256;$before=Fingerprint
    $execution=Read-Own-Execution
    if($execution.version -ne 3){throw 'Expected worker lifecycle v3'}
    $probeRoot=Join-Path $env:TEMP ('kraqel-isolation-boot-'+([guid]$execution.probeId).ToString('N'))
    $spawn=Get-Content -LiteralPath "$probeRoot/spawn.json" -Raw | ConvertFrom-Json
    $parent=Get-Content -LiteralPath "$probeRoot/parent.json" -Raw | ConvertFrom-Json
    if($spawn.pid -ne $parent.pid -or $parent.singleProcessLimit -ne 1 -or -not $parent.leaseHeld -or -not $parent.token.expectedIdentity -or $parent.token.capabilityCount -ne 0){throw 'Native worker checks missing'}
    Confirm-Own-Fixture-End $spawn
    $moniker='org.kraqel.probe.boot-'+([guid]$execution.probeId).ToString('N')
    if(-not (Profile-Exists $moniker)){throw 'Crash profile unexpectedly missing'}
    Invoke-Step reconcile $boundHead 1;Invoke-Step start $pendingHead 1
    if((Fingerprint) -ne $before -or -not (Profile-Exists $moniker)){throw 'Stale/pending request mutated state'}
    $manual=0;$blocked=($point -eq 'before-bound')
    if($blocked) {
        Invoke-Step reconcile $pendingHead 1
        if((Fingerprint) -ne $before -or -not (Profile-Exists $moniker)){throw 'Unbound worker was adopted'}
        # The controlled harness owns this just-created fixture and independently checked its
        # recorded native generation ended. This diagnostic evidence is never used by the broker.
        if([KraqelResourceFixtureCleanup]::DeleteAppContainerProfile($moniker) -lt 0){throw 'Own fixture cleanup failed'}
        if(Profile-Exists $moniker){throw 'Own fixture remains'};$manual=1
        Invoke-Step reconcile $pendingHead 1;Invoke-Step start $pendingHead 1
        if((Fingerprint) -ne $before){throw 'Unbound journal was reset or repaired'}
    } else {
        $state=Invoke-Step reconcile $pendingHead
        if($state.probePending -or $state.workspace.pendingLaunch -or -not $state.outcome.resourceReconciled -or (Profile-Exists $moniker)){throw 'Worker recovery incomplete'}
        Invoke-Step reconcile $state.workspace.control.headSha256 1
        $before=Fingerprint;Invoke-Step replay $state.workspace.control.headSha256 1 -Record
        if((Fingerprint) -ne $before){throw 'Worker replay changed control'}
        $state=Invoke-Step start $state.workspace.control.headSha256
        if($state.workspace.generation -ne 2 -or -not $state.outcome.authenticatedReceipt){throw 'Next worker generation failed'}
    }
    $state=Invoke-Step status $zero
    if([bool]$state.probePending -ne $blocked -or (Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $key -or (Get-FileHash -LiteralPath "$store/data/workspace/sentinel.txt").Hash -ne $sentinel){throw 'Final state/key/workspace mismatch'}
    $cases.Add(@{point=$point;localStoreRoot=$store;final=$state;blockedUnboundWorker=$blocked;keyAndWorkspacePreserved=$true;replayAndStaleBlocked=$true;harnessOwnFixtureCleanup=$manual})
}
@{passed=$true;nativePreviewOnly=$true;productionAcceptance=$false;controllerSha256=$ProbeSha256;uiFixtureSha256=$UiSha256;controllerProcesses=$steps.Count;cases=$cases;uiStarts=0;importTasksCreated=0;networkMeasurement=$false} | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath "$fixtureRoot/result.json"
Write-Output $fixtureRoot
