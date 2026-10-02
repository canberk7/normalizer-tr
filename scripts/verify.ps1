[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PythonPath,
    [Parameter(Mandatory)][string]$OutputDirectory,
    [string]$RustupHome
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = Split-Path $PSScriptRoot -Parent
$oldLocation = Get-Location
$saved = @{}
foreach ($name in @('RUSTUP_HOME', 'PYO3_PYTHON', 'PYTHONIOENCODING', 'RUSTDOCFLAGS')) {
    $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
}
$stages = [System.Collections.Generic.List[object]]::new()
$createdOutput = $false

function Invoke-Step([string]$Name, [string]$Executable, [string[]]$Arguments) {
    $log = Join-Path $OutputDirectory "logs\$Name.log"
    Write-Host "Running $Name"
    & $Executable @Arguments *> $log
    $code = $LASTEXITCODE
    $status = if ($code -eq 0) {'passed'} else {'failed'}
    $stages.Add([ordered]@{name=$Name; status=$status; exit_code=$code; log=$log})
    if ($code -ne 0) {throw "Verification stage '$Name' failed; see $log"}
}

try {
    Set-Location $root
    if (git status --porcelain) {throw 'Final verification requires a clean local source revision.'}
    if (-not [IO.Path]::IsPathFullyQualified($OutputDirectory)) {throw 'OutputDirectory must be absolute.'}
    $OutputDirectory = [IO.Path]::GetFullPath($OutputDirectory)
    if ($OutputDirectory.StartsWith("$root\", [StringComparison]::OrdinalIgnoreCase) -and
        -not $OutputDirectory.StartsWith("$root\target\", [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Inside the worktree, output is allowed only under ignored target\.'
    }
    $ancestor = [IO.DirectoryInfo]::new($OutputDirectory).Parent
    while ($ancestor) {
        if ($ancestor.Exists -and ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw 'Output symlink/junction ancestors are not accepted.'
        }
        $ancestor = $ancestor.Parent
    }
    if (Test-Path -LiteralPath $OutputDirectory) {throw 'OutputDirectory must be new; no overwrites.'}
    $PythonPath = (Resolve-Path -LiteralPath $PythonPath).Path
    if ($RustupHome) {$env:RUSTUP_HOME = (Resolve-Path -LiteralPath $RustupHome).Path}
    $env:PYO3_PYTHON = $PythonPath
    $env:PYTHONIOENCODING = 'utf-8'
    $env:RUSTDOCFLAGS = '-D warnings'
    New-Item -ItemType Directory -Path $OutputDirectory | Out-Null
    $createdOutput = $true
    foreach ($directory in @('logs','reports','packages')) {
        New-Item -ItemType Directory -Path (Join-Path $OutputDirectory $directory) | Out-Null
    }
    $hostRecord = @{
        cpu=(Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors,LoadPercentage)
        os=(Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,OSArchitecture)
        power_scheme=((powercfg /getactivescheme | Out-String).Trim())
        capture_utc=((Get-Date).ToUniversalTime().ToString('o'))
        shared_host=$true; settings_changed=$false; load_sample='setup snapshot, not each individual call'
    }
    $hostRecord | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $OutputDirectory 'host.json') -Encoding utf8
    Invoke-Step 'fmt' 'cargo' @('fmt','--all','--check')
    Invoke-Step 'clippy' 'cargo' @('clippy','--locked','--workspace','--all-targets','--all-features','--','-D','warnings')
    Invoke-Step 'test-default' 'cargo' @('test','--locked')
    Invoke-Step 'test-no-default' 'cargo' @('test','--locked','--no-default-features')
    Invoke-Step 'test-serde' 'cargo' @('test','--locked','--all-features')
    Invoke-Step 'docs-default' 'cargo' @('doc','--locked','--no-deps')
    Invoke-Step 'docs-no-default' 'cargo' @('doc','--locked','--no-deps','--no-default-features')
    Invoke-Step 'docs-serde' 'cargo' @('doc','--locked','--no-deps','--all-features')
    Invoke-Step 'ruff' $PythonPath @('-m','ruff','check','bindings\python','benches','scripts')
    Invoke-Step 'python-format' $PythonPath @('-m','ruff','format','--check','bindings\python','benches','scripts')
    Invoke-Step 'typing' $PythonPath @('-m','mypy','bindings\python\python','--check-untyped-defs')
    Invoke-Step 'verification-machinery' $PythonPath @('scripts\test_verification.py')
    $audit = Join-Path $root 'target\audit-tool\bin\cargo-audit.exe'
    if (-not (Test-Path -LiteralPath $audit)) {throw 'Missing local cargo-audit tool; provision it in the isolated target tool directory.'}
    Invoke-Step 'rust-advisories' $audit @('audit','--db',(Join-Path $root 'target\advisory-db'),'--no-fetch','--format','json')
    $site = Join-Path (Split-Path (Split-Path $PythonPath -Parent) -Parent) 'Lib\site-packages'
    Invoke-Step 'python-advisories' $PythonPath @('-m','pip_audit','--path',$site,'--cache-dir',(Join-Path $OutputDirectory 'audit-cache'),'--format','json')
    Invoke-Step 'example-current' 'cargo' @('run','--locked','--example','normalize')
    Invoke-Step 'example-coverage' 'cargo' @('run','--locked','--example','general')
    Invoke-Step 'package-core' 'cargo' @('package','-p','normalizer-tr','--locked')
    Invoke-Step 'prepare-core-consumer' $PythonPath @('scripts\verification.py','prepare',$OutputDirectory)
    $prepared = Get-Content -LiteralPath (Join-Path $OutputDirectory 'prepared.json') -Raw | ConvertFrom-Json
    $consumerManifest = $prepared.consumer_manifest
    Invoke-Step 'consumer-lock' 'cargo' @('generate-lockfile','--manifest-path',$consumerManifest)
    Invoke-Step 'consumer-default' 'cargo' @('test','--locked','--manifest-path',$consumerManifest)
    Invoke-Step 'consumer-serde' 'cargo' @('test','--locked','--all-features','--manifest-path',$consumerManifest)
    Invoke-Step 'packaged-tests' 'cargo' @('test','--locked','--all-features','--manifest-path',$prepared.package_manifest)
    Invoke-Step 'build-wheel' $PythonPath @('-m','maturin','build','--release','--locked','--manifest-path','bindings\python\Cargo.toml','--interpreter',$PythonPath,'--out',(Join-Path $OutputDirectory 'packages'))
    Invoke-Step 'inspect-wheel' $PythonPath @('scripts\verification.py','wheel',$OutputDirectory)
    Invoke-Step 'fresh-python-env' $PythonPath @('-m','venv',(Join-Path $OutputDirectory 'py'))
    $consumerPython = Join-Path $OutputDirectory 'py\Scripts\python.exe'
    $wheel = (Get-ChildItem (Join-Path $OutputDirectory 'packages') -Filter '*.whl').FullName
    Invoke-Step 'install-wheel' $consumerPython @('-m','pip','install','--no-deps',$wheel)
    Invoke-Step 'install-test-tool' $consumerPython @('-m','pip','install','pytest==9.0.3')
    Invoke-Step 'installed-python-tests' $consumerPython @('-m','pytest','bindings\python\tests','-q')
    Invoke-Step 'python-independent-consumer' $consumerPython @('scripts\verification.py','consume',$OutputDirectory)
    foreach ($repeat in 1..3) {
        Invoke-Step "latency-$repeat" 'cargo' @('bench','--locked','--features','serde','--bench','latency','--','--output',(Join-Path $OutputDirectory "reports\rust-$repeat.json"))
    }
    Invoke-Step 'python-latency' $consumerPython @('benches\python_latency.py','--output',(Join-Path $OutputDirectory 'reports\python.json'))
    $stages | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $OutputDirectory 'stages.json') -Encoding utf8
    Invoke-Step 'final-manifest' $consumerPython @('scripts\verification.py','finalize',$OutputDirectory)
    Write-Host "Verification passed: $(Join-Path $OutputDirectory 'manifest.json')"
}
catch {
    if ($createdOutput) {
        @{status='failed'; error=$_.Exception.Message; stages=$stages} |
            ConvertTo-Json -Depth 8 | Set-Content (Join-Path $OutputDirectory 'failure.json') -Encoding utf8
    }
    Write-Error $_
    exit 1
}
finally {
    foreach ($name in $saved.Keys) {[Environment]::SetEnvironmentVariable($name,$saved[$name],'Process')}
    Set-Location $oldLocation
}
