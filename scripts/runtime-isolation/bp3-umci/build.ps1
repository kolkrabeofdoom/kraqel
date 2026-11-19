param([Parameter(Mandatory)][string]$OutputDirectory)

$ErrorActionPreference = 'Stop'
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $output) { throw 'Choose a fresh output directory.' }
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $vs) { throw 'MSVC x64 Build Tools required.' }
$module = Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll'
Import-Module $module
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
$null = New-Item -ItemType Directory -Path $output
$null = New-Item -ItemType Directory -Path (Join-Path $output 'allow')
$null = New-Item -ItemType Directory -Path (Join-Path $output 'deny')

function Build-Image([string]$Source, [string]$Target, [string[]]$Options, [string[]]$LinkOptions) {
    $binary = Join-Path $output $Target
    $object = "$binary.obj"
    & cl.exe /nologo /W4 /WX /O2 /MT /utf-8 "/Fo$object" "/Fe$binary" @Options (Join-Path $PSScriptRoot $Source) @LinkOptions
    if ($LASTEXITCODE -ne 0) { throw "Compilation failed for $Target." }
    Remove-Item -LiteralPath $object
}
Build-Image 'starter.c' 'allow/bp3-starter.exe' @('userenv.lib','advapi32.lib')
# A console worker can request conhost.exe before wmain; leaf roles cannot start any child.
Build-Image 'worker.c' 'allow/bp3-worker.exe' @('advapi32.lib') @('/link','/SUBSYSTEM:WINDOWS','/ENTRY:wmainCRTStartup')
Build-Image 'worker.c' 'deny/foreign.exe' @('/DBP3_FOREIGN')
Build-Image 'marker.c' 'allow/allowed.dll' @('/LD')
Build-Image 'marker.c' 'deny/foreign.dll' @('/LD','/DBP3_FOREIGN')
# Two ordinary PE import tables, one for each independently built DLL.
foreach ($case in 'allowed','foreign') {
    $importDir = if ($case -eq 'allowed') { 'allow' } else { 'deny' }
    $library = Join-Path $output "$importDir/$case.lib"
    Build-Image 'static-import.c' "allow/bp3-static-$case.exe" @($library) @('/link','/SUBSYSTEM:WINDOWS','/ENTRY:wmainCRTStartup')
}
foreach ($file in @('allow/allowed.lib','allow/allowed.exp','deny/foreign.lib','deny/foreign.exp')) {
    Remove-Item -LiteralPath (Join-Path $output $file)
}
Set-Content -LiteralPath (Join-Path $output 'input.bin') -Value 'K' -NoNewline -Encoding ascii
$files = foreach ($name in @('allow/bp3-starter.exe','allow/bp3-worker.exe','allow/allowed.dll',
                           'allow/bp3-static-allowed.exe','allow/bp3-static-foreign.exe',
                           'deny/foreign.exe','deny/foreign.dll','input.bin')) {
    $item = Get-Item -LiteralPath (Join-Path $output $name)
    [ordered]@{ path = $name; bytes = $item.Length; sha256 = (Get-FileHash -LiteralPath $item.FullName).Hash }
}
[ordered]@{ version = 2; purpose = 'bp3-umci-synthetic-fixture'; files = @($files) } |
    ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $output 'manifest.json') -Encoding utf8
