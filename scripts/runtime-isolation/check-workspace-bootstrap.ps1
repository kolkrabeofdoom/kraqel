param(
    [Parameter(Mandatory=$true)][string]$Probe,
    [Parameter(Mandatory=$true)][string]$ProbeSha256,
    [Parameter(Mandatory=$true)][string]$UiFixture,
    [Parameter(Mandatory=$true)][string]$UiSha256
)
# P2b: six synthetic workers, no GUI/import/network diagnostic. Raw evidence stays in TEMP.
$ErrorActionPreference='Stop'
foreach($pair in @(@($Probe,$ProbeSha256),@($UiFixture,$UiSha256))) {
    if($pair[1] -cnotmatch '^[0-9a-f]{64}$' -or (Get-FileHash -LiteralPath $pair[0]).Hash.ToLowerInvariant() -ne $pair[1]){throw 'Unexpected explicitly approved program hash'}
}
$Probe=(Resolve-Path -LiteralPath $Probe).Path;$UiFixture=(Resolve-Path -LiteralPath $UiFixture).Path
$fixtureRoot=Join-Path $env:TEMP ('kraqel-isolation-controller-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
$steps=[Collections.Generic.List[object]]::new();$cases=[Collections.Generic.List[object]]::new();$zero='0'*64
function Invoke-Step([string]$Action,[string]$Head,[int]$Expected=0,[switch]$Record) {
    $info=[Diagnostics.ProcessStartInfo]::new($Probe)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    $argv=if($Record){@('product-workspace-child',$store,$Action,$UiFixture,$UiSha256,$Head)}else{@('product-workspace-probe-child',$store,$Action,$Head)}
    foreach($arg in $argv){$info.ArgumentList.Add($arg)}
    $process=[Diagnostics.Process]::Start($info)
    try {
        $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
        if(-not $process.WaitForExit(30000)){$process.Kill();$process.WaitForExit();throw 'Bootstrap check timed out'}
        $out=$stdout.GetAwaiter().GetResult();$err=$stderr.GetAwaiter().GetResult();$index=$steps.Count
        [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stdout"),$out);[IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stderr"),$err)
        $steps.Add(@{pid=$process.Id;case=$caseName;action=$Action;record=[bool]$Record;exitCode=$process.ExitCode;expectedExitCode=$Expected})
        $steps | ConvertTo-Json | Set-Content -LiteralPath "$fixtureRoot/processes.json"
        if($process.ExitCode -ne $Expected){throw 'Unexpected exit; inspect private evidence'}
        if($Expected -eq 0){return $out | ConvertFrom-Json}
    } finally {$process.Dispose()}
}
function Fingerprint {
    return (@(Get-ChildItem -LiteralPath "$store/control/records" -File | Sort-Object Name | ForEach-Object {$_.Name+':'+(Get-FileHash -LiteralPath $_.FullName).Hash}) -join '|')
}
foreach($mode in @('start','start-unassigned','start-bad-mac')) {
    $caseName='case-'+$cases.Count.ToString('D2');$caseRoot=Join-Path $fixtureRoot $caseName
    New-Item -ItemType Directory -Path $caseRoot | Out-Null;$store=Join-Path $caseRoot 'store'
    $state=Invoke-Step init $zero -Record
    $state=Invoke-Step migrate5 $state.control.headSha256 -Record
    $state=Invoke-Step bind $state.control.headSha256 -Record
    $key=(Get-FileHash -LiteralPath "$store/control/authority.key").Hash
    $sentinel=(Get-FileHash -LiteralPath "$store/data/workspace/sentinel.txt").Hash
    $boundHead=$state.control.headSha256
    if($mode -eq 'start'){$state=Invoke-Step $mode $boundHead}else{Invoke-Step $mode $boundHead 1}
    $state=Invoke-Step status $zero
    if($state.probePending -or $state.workspace.pendingLaunch -or $state.workspace.generation -ne 1){throw 'First worker did not settle'}
    $fingerprint=Fingerprint
    Invoke-Step replay $state.workspace.control.headSha256 1 -Record
    Invoke-Step start $boundHead 1
    if((Fingerprint) -ne $fingerprint){throw 'Replay/stale operation changed records'}
    $state=Invoke-Step start $state.workspace.control.headSha256
    if($state.probePending -or $state.workspace.pendingLaunch -or $state.workspace.generation -ne 2 -or -not $state.outcome.authenticatedReceipt -or $state.outcome.uiStarted -or $state.outcome.importTaskCreated){throw 'Second worker did not succeed'}
    $state=Invoke-Step status $zero
    if((Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $key -or (Get-FileHash -LiteralPath "$store/data/workspace/sentinel.txt").Hash -ne $sentinel){throw 'Private workspace/key changed'}
    $ends=@(Get-ChildItem -LiteralPath $caseRoot -Filter 'probe-*-end.json' | ForEach-Object {Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json})
    if($ends.Count -ne 2 -or @($ends | Where-Object {-not $_.profileRemoved}).Count -ne 0){throw 'Profile cleanup incomplete'}
    $cases.Add(@{mode=$mode;localStoreRoot=$store;final=$state;ends=$ends;keyUnchanged=$true;sentinelUnchanged=$true;replayAndStaleBlocked=$true})
}
@{passed=$true;nativePreviewOnly=$true;productionAcceptance=$false;controllerSha256=$ProbeSha256;uiFixtureSha256=$UiSha256;controllerProcesses=$steps.Count;cases=$cases;uiStarts=0;importTasksCreated=0;networkMeasurement=$false} | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath "$fixtureRoot/result.json"
Write-Output $fixtureRoot
