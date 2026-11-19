param(
    [Parameter(Mandatory)][string]$FixtureDirectory,
    [Parameter(Mandatory)][string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
$baseline = Join-Path $env:SystemRoot 'schemas\CodeIntegrity\ExamplePolicies\DefaultWindows_Enforced.xml'
$allowed = Join-Path $FixtureDirectory 'allow'
$expected = @('allowed.dll','bp3-starter.exe','bp3-worker.exe')
$version = (Get-Content -LiteralPath (Join-Path $FixtureDirectory 'manifest.json') -Raw | ConvertFrom-Json).version
if ($version -eq 2) { $expected += 'bp3-static-allowed.exe','bp3-static-foreign.exe' }
elseif ($version -ne 1) { throw 'Unknown fixture manifest version.' }
$files = @(Get-ChildItem -LiteralPath $allowed -Force)
if ($files.Count -ne $expected.Count -or
    @($files | Where-Object {
        $_.PSIsContainer -or $_.Name -cnotin $expected -or
        ($_.Attributes -band [IO.FileAttributes]::ReparsePoint)
    }).Count -ne 0) {
    throw 'Only the expected built positive fixtures belong in the allow directory.'
}
if (Test-Path -LiteralPath $OutputDirectory) { throw 'Choose a fresh policy output directory.' }
$out = (New-Item -ItemType Directory -Path $OutputDirectory).FullName
$hashXml = Join-Path $out 'test-image-hashes.xml'
$policyXml = Join-Path $out 'policy.xml'
$baselineCopy = Join-Path $out 'platform-template.xml'
$baselineHash = (Get-FileHash -LiteralPath $baseline).Hash
Copy-Item -LiteralPath $baseline -Destination $baselineCopy
if ((Get-FileHash -LiteralPath $baselineCopy).Hash -ne $baselineHash) {
    throw 'Windows platform template copy differs.'
}
New-CIPolicy -ScanPath $allowed -FilePath $hashXml -Level Hash -UserPEs -MultiplePolicyFormat -NoShadowCopy
Merge-CIPolicy -PolicyPaths @($baselineCopy,$hashXml) -OutputFilePath $policyXml | Out-Null
Set-CIPolicyIdInfo -FilePath $policyXml -ResetPolicyID -PolicyName 'KRAQEL BP3 Offline Guest Probe' | Out-Null
[xml]$xml = Get-Content -LiteralPath $policyXml -Raw
foreach ($option in @{ 3 = 'Enabled:Audit Mode'; 13 = 'Enabled:Managed Installer'; 14 = 'Enabled:Intelligent Security Graph Authorization' }.GetEnumerator()) {
    if ($option.Value -in $xml.SiPolicy.Rules.Rule.Option) {
        Set-RuleOption -FilePath $policyXml -Option $option.Key -Delete
    }
}
Set-RuleOption -FilePath $policyXml -Option 0
[xml]$xml = Get-Content -LiteralPath $policyXml -Raw
if ('Enabled:UMCI' -notin $xml.SiPolicy.Rules.Rule.Option -or
    'Enabled:Audit Mode' -in $xml.SiPolicy.Rules.Rule.Option) {
    throw 'Generated policy options did not match expected enforcement.'
}
$id = [guid]$xml.SiPolicy.PolicyID
$binary = Join-Path $out ('{' + $id + '}.cip')
ConvertFrom-CIPolicy -XmlFilePath $policyXml -BinaryFilePath $binary
[ordered]@{
    policyId = $id
    policySha256 = (Get-FileHash -LiteralPath $binary).Hash
    platformTemplateSha256 = $baselineHash
    fixtureManifestSha256 = (Get-FileHash -LiteralPath (Join-Path $FixtureDirectory 'manifest.json')).Hash
    platformBuild = (Get-CimInstance Win32_OperatingSystem).Version
    deployed = $false
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $out 'policy-manifest.json') -Encoding utf8
