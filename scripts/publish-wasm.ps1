<#
.SYNOPSIS
  Build and publish @faicad/brepkit2-wasm to npm (single-package script).

.DESCRIPTION
  Delegates the heavy lifting to the existing xtask pipeline
  (`cargo xtask wasm-publish`: dual-target wasm-pack build -> wasm-opt ->
  pkg merge -> output validation -> node smoke test), then applies the
  publish-side guards borrowed from faijs scripts/publish-all.ps1:
  version consistency check, tarball white-list assertion, npm publish,
  post-publish registry verification, and a publish record.

  Unlike the faijs monorepo, this repo publishes exactly one package:
  @faicad/brepkit2-wasm (see xtask/src/wasm.rs::NPM_PKG_NAME).

.PARAMETER AutoVersion
  Compute the next version from git history (conventional commits) via
  `cargo xtask wasm-release`, bump crates/wasm/Cargo.toml, and regenerate the
  CHANGELOG Unreleased section — then build and publish. Use this for normal
  releases; without it, the script publishes the version already in
  crates/wasm/Cargo.toml.

.PARAMETER AllowMajor
  With -AutoVersion: allow a breaking change to bump the major version.
  Default clamps major -> minor (the fork inherits upstream's version line).

.PARAMETER Version
  With -AutoVersion: override the computed version with an explicit X.Y.Z.

.PARAMETER DryRun
  Build + validate + pack-assert only; do NOT actually publish.

.PARAMETER Tag
  npm dist-tag, default "latest". For a first release use "next" and promote
  after smoke-testing.

.PARAMETER SkipBuild
  Skip `cargo xtask wasm-build` (use when pkg/ is already built and current).
  The publish flow still validates the existing pkg/ output.

.PARAMETER SkipSmoke
  Skip the node smoke test (pkg validation still runs).

.PARAMETER Provenance
  Add --provenance. Requires an OIDC CI environment (GitHub Actions); do NOT
  use locally — the publish will fail without a trusted OIDC token.

.PARAMETER Otp
  npm 2FA one-time password, if your account has 2FA enabled without an agent.

.EXAMPLE
  .\scripts\publish-wasm.ps1 -DryRun               # local dry run, no publish
  .\scripts\publish-wasm.ps1 -AutoVersion -DryRun  # dry run with version bump + CHANGELOG preview
  .\scripts\publish-wasm.ps1 -AutoVersion          # normal release: bump + CHANGELOG + publish
  .\scripts\publish-wasm.ps1 -Tag next             # first release to "next"
  .\scripts\publish-wasm.ps1 -Otp 123456           # real publish with 2FA
#>

param(
  [switch]$AutoVersion,
  [switch]$AllowMajor,
  [string]$Version,
  [switch]$DryRun,
  [string]$Tag = 'latest',
  [switch]$SkipBuild,
  [switch]$SkipSmoke,
  [switch]$Provenance,
  [string]$Otp
)

$ErrorActionPreference = 'Stop'

# Resolve npm invoker. Windows note: resolve to npm.cmd, not the PowerShell
# shim (npm.ps1) — invoking the .ps1 shim from pwsh can drop script args.
if ($env:NPM_CLI_NODE -and $env:NPM_CLI_PATH) {
  $npm = { & $env:NPM_CLI_NODE $env:NPM_CLI_PATH @args }
} elseif ($IsWindows -or $env:OS -eq 'Windows_NT') {
  $npmCmd = Get-Command npm.cmd -ErrorAction SilentlyContinue
  if (-not $npmCmd) { $npmCmd = Get-Command npm -ErrorAction SilentlyContinue }
  if (-not $npmCmd) { throw 'npm not found on PATH' }
  $npmExe = $npmCmd.Source
  $npm = { & $npmExe @args }
} else {
  $npm = { & npm @args }
}

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$PkgDir = Join-Path $RepoRoot 'crates/wasm/pkg'
$PkgName = '@faicad/brepkit2-wasm'

if (-not (Test-Path (Join-Path $RepoRoot 'xtask/Cargo.toml'))) {
  throw "xtask not found at $RepoRoot/xtask — run from within the brepkit2 repo"
}

function Run-Xtask {
  param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Args)
  Push-Location $RepoRoot
  try { cargo xtask @Args 2>&1 | ForEach-Object { Write-Host $_ } ; if ($LASTEXITCODE -ne 0) { throw "cargo xtask exit code $LASTEXITCODE" } }
  finally { Pop-Location }
}

# ---------------------------------------------------------------------------
# Step 1: build (dual target: bundler + node), optimize, merge, validate.
# With -AutoVersion this becomes the full wasm-release flow: compute the next
# version from git history, bump crates/wasm/Cargo.toml, regenerate the
# CHANGELOG Unreleased section, then build/validate/smoke/publish in xtask.
# ---------------------------------------------------------------------------
if ($AutoVersion) {
  if ($SkipBuild) { Write-Warning '-SkipBuild ignored with -AutoVersion (the release flow always builds)' }
  Write-Host '-> [1/4] auto-version release (cargo xtask wasm-release) ...' -ForegroundColor Cyan
  $relArgs = @('wasm-release')
  if ($DryRun) { $relArgs += '--dry-run' }
  if ($AllowMajor) { $relArgs += '--allow-major' }
  if ($Version) { $relArgs += @('--version', $Version) }
  Run-Xtask @relArgs
  Write-Host '   OK release flow complete' -ForegroundColor Green
} elseif (-not $SkipBuild) {
  Write-Host '-> [1/4] building WASM package (cargo xtask wasm-build) ...' -ForegroundColor Cyan
  Run-Xtask 'wasm-build'
  Write-Host '   OK build complete' -ForegroundColor Green
} else {
  Write-Host '-> [1/4] skipping build (-SkipBuild); validating existing pkg/ ...' -ForegroundColor Yellow
}

# ---------------------------------------------------------------------------
# Step 2: smoke test (optional but strongly recommended for a real publish).
# ---------------------------------------------------------------------------
if (-not $SkipSmoke) {
  Write-Host '-> [2/4] running smoke test (node scripts/test-wasm-smoke.mjs) ...' -ForegroundColor Cyan
  Push-Location $RepoRoot
  try {
    node (Join-Path $RepoRoot 'scripts/test-wasm-smoke.mjs') 2>&1 | ForEach-Object { Write-Host $_ }
    if ($LASTEXITCODE -ne 0) { throw "smoke test failed (exit $LASTEXITCODE)" }
  } finally { Pop-Location }
  Write-Host '   OK smoke test passed' -ForegroundColor Green
} else {
  Write-Host '-> [2/4] skipping smoke test (-SkipSmoke)' -ForegroundColor Yellow
}

# ---------------------------------------------------------------------------
# Step 3: pack white-list assertion (E2 pattern from faijs publish-all.ps1).
# Must run npm pack from inside the package dir with '.' — `npm pack <path>`
# from a parent dir makes npm parse the path as a package *spec*.
# ---------------------------------------------------------------------------
Write-Host '-> [3/4] npm pack --dry-run + white-list assertion ...' -ForegroundColor Cyan
Push-Location $PkgDir
try {
  $packJson = & $npm pack --dry-run --json . 2>&1 | Out-String
  $packExit = $LASTEXITCODE
} finally { Pop-Location }
if ($packExit -ne 0) { throw "npm pack failed (exit $packExit):`n$packJson" }

$m = [regex]::Match($packJson, '\[.*\]|\{.*\}', [System.Text.RegularExpressions.RegexOptions]::Singleline)
if (-not $m.Success) { throw "Cannot parse npm pack output as JSON:`n$packJson" }
$data = $m.Value | ConvertFrom-Json
if ($data -is [System.Array]) { $data = $data[0] }
$files = $data.files.path

# The tarball must contain ONLY the build artifacts declared in package.json
# "files" — no sources, tests, target/, or pkg-node leftovers.
$DenyPatterns = @(
  '(^|/)src/', '(^|/)tests?/', '(^|/)target/', '(^|/)pkg-node/', 'node_modules/',
  '\.test\.', '\.spec\.', '\.md$', '\.log$', 'LICENSE-', '\.rs$'
)
$bad = @()
foreach ($f in $files) {
  foreach ($d in $DenyPatterns) { if ($f -match $d) { $bad += $f; break } }
}
if ($bad.Count -gt 0) { throw "tarball contains forbidden files: $($bad -join ', ')" }
if (-not ($files -contains 'package.json')) { throw 'tarball missing package.json' }
if (-not ($files -contains 'brepkit_wasm_bg.wasm')) { throw 'tarball missing brepkit_wasm_bg.wasm (empty build?)' }
if (-not ($files -contains 'brepkit_wasm.d.ts')) { throw 'tarball missing brepkit_wasm.d.ts' }

$PkgJsonPath = Join-Path $PkgDir 'package.json'
$PkgJson = Get-Content $PkgJsonPath -Raw | ConvertFrom-Json
if ($PkgJson.name -ne $PkgName) { throw "package.json name is '$($PkgJson.name)', expected '$PkgName'" }
$Version = $PkgJson.version
if (-not $Version) { throw 'package.json has no version' }
Write-Host "   OK white-list passed ($($files.Count) files, $([math]::Round($data.unpackedSize/1024, 1)) KB), version $Version" -ForegroundColor Green

# ---------------------------------------------------------------------------
# Step 4: publish + verify + record.
# ---------------------------------------------------------------------------
if (-not $DryRun) {
  $pubArgs = @('publish', '--access', 'public')
  if ($Tag -ne 'latest') { $pubArgs += '--tag'; $pubArgs += $Tag }
  if ($Provenance) { $pubArgs += '--provenance' }
  $pubArgs += '--registry'; $pubArgs += 'https://registry.npmjs.org/'
  if ($Otp) { $pubArgs += '--otp'; $pubArgs += $Otp }

  Write-Host "-> [4/4] publishing $PkgName@$Version (tag=$Tag) ..." -ForegroundColor Cyan
  Push-Location $PkgDir
  try {
    $pubOut = & $npm @pubArgs 2>&1 | Out-String
    if ($pubOut) { Write-Host $pubOut.TrimEnd() }
  } finally { Pop-Location }
  if ($LASTEXITCODE -ne 0) { throw 'npm publish failed' }
} else {
  Write-Host '-> [4/4] DRY-RUN: skipping publish' -ForegroundColor Yellow
}

# Post-publish verification (warn-only; npm CDN lag is not a publish failure).
if (-not $DryRun) {
  Write-Host '-> verifying published package ...' -ForegroundColor Cyan
  $encoded = [uri]::EscapeDataString($PkgName)
  $verUrl = "https://registry.npmjs.org/$encoded/$Version"
  $published = $null
  for ($attempt = 1; $attempt -le 5; $attempt++) {
    $json = & curl.exe -s --max-time 30 -H 'Accept-Encoding: identity' $verUrl 2>$null
    if ($LASTEXITCODE -eq 0 -and $json) {
      try {
        $resp = $json | ConvertFrom-Json
        if ($resp.name -eq $PkgName -and $resp.version -eq $Version) { $published = $Version; break }
      } catch { }
    }
    Start-Sleep -Seconds 5
  }
  if ($published -eq $Version) {
    Write-Host "   OK verified $PkgName@$Version" -ForegroundColor Green
  } else {
    Write-Warning "could not verify $Version on the registry yet (CDN lag); check later with: npm view $PkgName version"
  }
}

# Publish record.
$record = [ordered]@{
  name        = $PkgName
  version     = $Version
  tag         = $Tag
  dryRun      = [bool]$DryRun
  publishedAt = (Get-Date -UFormat '%Y-%m-%dT%H:%M:%S%z')
}
$recordPath = Join-Path $RepoRoot "out-publish-$(Get-Date -UFormat '%Y%m%d-%H%M%S').json"
$record | ConvertTo-Json | Set-Content $recordPath
Write-Host "OK publish record: $recordPath" -ForegroundColor Green
if ($DryRun) { Write-Host '(DRY-RUN: nothing published. Re-run without -DryRun to publish.)' -ForegroundColor Yellow }
