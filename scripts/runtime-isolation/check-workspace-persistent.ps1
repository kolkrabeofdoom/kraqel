param(
    [Parameter(Mandatory=$true)][string]$Probe,
    [Parameter(Mandatory=$true)][string]$ProbeSha256
)
# Data-only C2: bounded fresh fixtures, explicit executable hash, no UI/network diagnostics.
$ErrorActionPreference='Stop'
if($ProbeSha256 -cnotmatch '^[0-9a-f]{64}$' -or (Get-FileHash -LiteralPath $Probe).Hash.ToLowerInvariant() -ne $ProbeSha256){throw 'Unexpected explicitly selected C2 program hash'}
$Probe=(Resolve-Path -LiteralPath $Probe).Path
$fixtureRoot=Join-Path $env:TEMP ('kraqel-isolation-controller-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $fixtureRoot | Out-Null
New-Item -ItemType Directory -Path (Join-Path $fixtureRoot 'case-01') | Out-Null
$store=Join-Path $fixtureRoot 'case-01/store'
$steps=[Collections.Generic.List[object]]::new();$zero='0'*64
function Invoke-Step([string]$Action,[string]$Head,[int]$Expected=0) {
    $info=[Diagnostics.ProcessStartInfo]::new($Probe)
    $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
    foreach($arg in @('product-p2c-data-child',$store,$Action,$Head)){$info.ArgumentList.Add($arg)}
    $process=[Diagnostics.Process]::Start($info)
    try {
        $stdout=$process.StandardOutput.ReadToEndAsync();$stderr=$process.StandardError.ReadToEndAsync()
        if(-not $process.WaitForExit(30000)){$process.Kill();$process.WaitForExit();throw 'C2 fixture deadline exceeded'}
        $out=$stdout.GetAwaiter().GetResult();$err=$stderr.GetAwaiter().GetResult();$index=$steps.Count
        [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stdout"),$out)
        [IO.File]::WriteAllText((Join-Path $fixtureRoot "$index.stderr"),$err)
        $steps.Add(@{pid=$process.Id;action=$Action;exitCode=$process.ExitCode;expectedExitCode=$Expected})
        $steps | ConvertTo-Json | Set-Content -LiteralPath "$fixtureRoot/processes.json"
        if($process.ExitCode -ne $Expected){throw 'Unexpected C2 exit; inspect private evidence'}
        if($Expected -eq 0){return $out | ConvertFrom-Json}
    } finally {$process.Dispose()}
}
function Fingerprint {
    $files=@(Get-ChildItem -LiteralPath "$store/control/records" -File | Sort-Object Name)+@(Get-Item -LiteralPath "$store/control/head.mac","$store/control/authority.key")
    return (@($files | ForEach-Object {$_.Name+':'+(Get-FileHash -LiteralPath $_.FullName).Hash}) -join '|')
}
function Read-Own-Binding {
    $domain=[Text.Encoding]::UTF8.GetBytes("KRAQEL-PRODUCT-CONTROL"+[char]0+[char]1)
    $wire=[IO.File]::ReadAllBytes("$store/control/records/0003.mac");$offset=$domain.Length+32
    if($wire.Length -le $offset -or [Convert]::ToHexString($wire[0..($domain.Length-1)]) -ne [Convert]::ToHexString($domain)){throw 'Unexpected C2 record domain'}
    $key=[IO.File]::ReadAllBytes("$store/control/authority.key");$mac=[Security.Cryptography.HMACSHA256]::new($key)
    try {$expected=$mac.ComputeHash([byte[]]($domain+$wire[$offset..($wire.Length-1)]))}finally{$mac.Dispose();[Array]::Clear($key)}
    if(-not [Security.Cryptography.CryptographicOperations]::FixedTimeEquals($expected,[byte[]]$wire[$domain.Length..($offset-1)])){throw 'C2 binding MAC invalid'}
    $record=[Text.Encoding]::UTF8.GetString($wire,$offset,$wire.Length-$offset) | ConvertFrom-Json
    if($record.action.kind -ne 'bindC2Workspace' -or $record.action.binding.controlSchema -ne 6){throw 'C2 binding absent'}
    return $record.action.binding
}
function Profile-Present([string]$Sid) {
    $path='Software\Classes\Local Settings\Software\Microsoft\Windows\CurrentVersion\AppContainer\Mappings\'+$Sid
    $key=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($path)
    if($null -eq $key){return $false}
    try{return $true}finally{$key.Dispose()}
}
$state=Invoke-Step init $zero;$initialHead=$state.control.headSha256
$state=Invoke-Step prepare $initialHead;$boundHead=$state.control.headSha256
if(-not $state.c2Bound -or -not $state.profileConfirmed -or $state.uiStarted -or $state.uiExecutable -or $state.uiAbiSupported -ne 0){throw 'C2 data-only boundary missing'}
$binding=Read-Own-Binding
$workspaceId=[Convert]::ToHexString([byte[]]$binding.workspaceId).ToLowerInvariant()
$dataRoot=Join-Path $env:TEMP ('kraqel-isolation-p2c-'+$workspaceId)
$document=Join-Path $dataRoot 'projects/synthetic.txt'
if(-not (Profile-Present $binding.profileSid)){throw 'Persistent C2 profile absent'}
$before=Fingerprint;$keyHash=(Get-FileHash -LiteralPath "$store/control/authority.key").Hash
$state=Invoke-Step verify $boundHead
$original=(Get-FileHash -LiteralPath $document).Hash
$state=Invoke-Step exercise $boundHead
$edited=(Get-FileHash -LiteralPath $document).Hash
if($original -eq $edited){throw 'Synthetic edit not persisted'}
$state=Invoke-Step verify $boundHead
if((Get-FileHash -LiteralPath $document).Hash -ne $edited -or -not (Profile-Present $binding.profileSid) -or (Fingerprint) -ne $before){throw 'C2 reopen changed binding, data or profile'}
Invoke-Step prepare $initialHead 1
Invoke-Step prepare $boundHead 1
Invoke-Step verify $initialHead 1
if((Fingerprint) -ne $before){throw 'Rejected C2 request mutated control'}
$state=Invoke-Step retire $boundHead
if(-not $state.profileRetired -or (Profile-Present $binding.profileSid) -or -not (Test-Path -LiteralPath $document)){throw 'Explicit retirement failed or deleted data'}
$state=Invoke-Step retired-status $state.control.headSha256
if((Get-FileHash -LiteralPath "$store/control/authority.key").Hash -ne $keyHash -or (Get-FileHash -LiteralPath $document).Hash -ne $edited){throw 'C2 key/data changed during retirement'}
@{passed=$true;nativePreviewOnly=$true;productionAcceptance=$false;controllerSha256=$ProbeSha256;controllerProcesses=$steps.Count;controlSchema=6;records=6;profileCreated=1;profileRemoved=1;dataPreserved=$true;staleAndRepeatBlocked=$true;uiStarts=0;importTasksCreated=0;networkMeasurement=$false} | ConvertTo-Json | Set-Content -LiteralPath "$fixtureRoot/result.json"
Write-Output $fixtureRoot
