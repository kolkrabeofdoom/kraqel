param(
    [Parameter(Mandatory=$true)][string]$Probe,
    [Parameter(Mandatory=$true)][string]$ProbeSha256,
    [Parameter(Mandatory=$true)][string]$UiFixture,
    [Parameter(Mandatory=$true)][string]$UiSha256
)
# P2a record-only research. No UI process/profile/task, no use of the normal preview store.
$ErrorActionPreference='Stop'
foreach($pair in @(@($Probe,$ProbeSha256),@($UiFixture,$UiSha256))) {
    if($pair[1] -cnotmatch '^[0-9a-f]{64}$' -or (Get-FileHash -LiteralPath $pair[0]).Hash.ToLowerInvariant() -ne $pair[1]){throw 'Unexpected explicitly approved program hash'}
}
$Probe=(Resolve-Path -LiteralPath $Probe).Path;$UiFixture=(Resolve-Path -LiteralPath $UiFixture).Path
$fixtureRoot=Join-Path $env:TEMP ('kraqel-isolation-controller-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$steps=[Collections.Generic.List[object]]::new();$held=[Collections.Generic.List[Diagnostics.Process]]::new();$cases=[Collections.Generic.List[object]]::new()
$zero='0'*64
function Invoke-Record([string]$Action,[string]$Head,[int]$ExpectedExit=0) {
    $info=[Diagnostics.ProcessStartInfo]::new($Probe)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    foreach($arg in @('product-workspace-child',$store,$Action,$UiFixture,$UiSha256,$Head)){$info.ArgumentList.Add($arg)}
    $process=[Diagnostics.Process]::Start($info);$null=$process.Handle;$held.Add($process)
    $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
    if(-not $process.WaitForExit(30000)){$process.Kill();$process.WaitForExit();throw 'Workspace record test timeout'}
    $out=$stdout.GetAwaiter().GetResult();$err=$stderr.GetAwaiter().GetResult();$index=$steps.Count
    [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stdout"),$out)
    [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stderr"),$err)
    $steps.Add(@{pid=$process.Id;case=$caseName;action=$Action;exitCode=$process.ExitCode;expectedExitCode=$ExpectedExit})
    $steps | ConvertTo-Json | Set-Content -LiteralPath "$fixtureRoot/processes.json"
    if($process.ExitCode -ne $ExpectedExit){throw 'Unexpected workspace record exit; inspect private evidence'}
    if($ExpectedExit -eq 0){return $out | ConvertFrom-Json}
}
function Control-Fingerprint {
    $entries=@(Get-ChildItem -LiteralPath "$store/control/records" -File | Sort-Object Name | ForEach-Object {$_.Name+':'+(Get-FileHash -LiteralPath $_.FullName).Hash})
    foreach($name in @('head.mac','authority.key')){$entries+=($name+':'+(Get-FileHash -LiteralPath "$store/control/$name").Hash)}
    return $entries -join '|'
}
try {
    foreach($point in @('before','after','tail')) {
        $caseName='case-'+$cases.Count.ToString('D2');$caseRoot=Join-Path $fixtureRoot $caseName
        New-Item -ItemType Directory -Path $caseRoot | Out-Null;$store=Join-Path $caseRoot 'store'
        $state=Invoke-Record init $zero
        $state=Invoke-Record migrate5 $state.control.headSha256
        $state=Invoke-Record bind $state.control.headSha256
        $boundHead=$state.control.headSha256
        $key=(Get-FileHash -LiteralPath "$store/control/authority.key").Hash
        $sentinel=(Get-FileHash -LiteralPath "$store/data/workspace/sentinel.txt").Hash
        Invoke-Record "commit-crash-$point" $boundHead 86
        if($point -eq 'tail') {
            $fingerprint=Control-Fingerprint
            Invoke-Record status $zero 1;Invoke-Record reconcile $boundHead 1;Invoke-Record status $zero 1
            if((Control-Fingerprint) -ne $fingerprint){throw 'Unpublished tail was modified or repaired'}
            $cases.Add(@{point=$point;localStoreRoot=$store;boundHead=$boundHead;blocked=$true;controlUnchanged=$true;keyUnchanged=$true;sentinelUnchanged=$true})
            continue
        }
        $state=Invoke-Record status $zero
        if($state.generation -ne [int]($point -eq 'after') -or [bool]$state.pendingLaunch -ne ($point -eq 'after')){throw 'Wrong reconstructed generation'}
        $fingerprint=Control-Fingerprint
        if($point -eq 'before') {
            if($state.control.headSha256 -ne $boundHead){throw 'Pre-record crash consumed control'}
            Invoke-Record commit $zero 1
            if((Control-Fingerprint) -ne $fingerprint){throw 'Stale commit mutated control'}
            $state=Invoke-Record commit $state.control.headSha256
        } else {
            Invoke-Record commit $state.control.headSha256 1
            Invoke-Record reconcile $boundHead 1
            if((Control-Fingerprint) -ne $fingerprint){throw 'Pending/stale operation mutated control'}
        }
        $state=Invoke-Record reconcile $state.control.headSha256
        $fingerprint=Control-Fingerprint
        Invoke-Record replay $state.control.headSha256 1
        if((Control-Fingerprint) -ne $fingerprint){throw 'Replay mutated control'}
        $state=Invoke-Record commit $state.control.headSha256
        $state=Invoke-Record reconcile $state.control.headSha256
        $state=Invoke-Record status $zero
        if($state.generation -ne 2 -or $state.pendingLaunch -or $state.uiStarted -or $state.profileCreated -or $state.importTaskCreated){throw 'Unexpected launch/task/profile authority'}
        if((Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $key -or (Get-FileHash -LiteralPath "$store/data/workspace/sentinel.txt").Hash -ne $sentinel){throw 'Key or workspace data changed'}
        $cases.Add(@{point=$point;localStoreRoot=$store;boundHead=$boundHead;blocked=$false;controlUnchanged=$true;keyUnchanged=$true;sentinelUnchanged=$true;final=$state})
    }
    @{passed=$true;nativePreviewOnly=$true;productionAcceptance=$false;controllerSha256=$ProbeSha256;uiFixtureSha256=$UiSha256;childProcesses=$steps.Count;cases=$cases;uiStarts=0;profilesCreated=0;importTasksCreated=0;networkMeasurement=$false} | ConvertTo-Json -Depth 15 | Set-Content -LiteralPath "$fixtureRoot/result.json"
    Write-Output $fixtureRoot
} finally {foreach($process in $held){$process.Dispose()}}
