# ============================================================
# Jarvis Assistant — Automated Android APK Builder
# Right-click → "Run with PowerShell"
# ============================================================

$ErrorActionPreference = "Stop"

function Log($msg) { Write-Host "`n[BUILD] $msg" -ForegroundColor Cyan }
function Ok($msg)  { Write-Host "[OK]    $msg" -ForegroundColor Green }
function Err($msg) {
    Write-Host "`n[ERROR] $msg" -ForegroundColor Red
    Write-Host "`nPress any key to close..." -ForegroundColor Yellow
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

# ── 0. Ensure running as admin ────────────────────────────────────────────────
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]"Administrator")) {
    Log "Restarting as Administrator..."
    Start-Process powershell "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs
    exit
}

# ── 1. Chocolatey ─────────────────────────────────────────────────────────────
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Log "Installing Chocolatey..."
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    $env:Path += ";$env:ProgramData\chocolatey\bin"
    Ok "Chocolatey installed"
} else {
    Ok "Chocolatey already installed"
}

# ── 2. Java 17 ────────────────────────────────────────────────────────────────
# Always resolve JAVA_HOME to the real folder (never a wildcard)
function Find-JavaHome {
    $adoptium = "C:\Program Files\Eclipse Adoptium"
    if (Test-Path $adoptium) {
        $jdk = Get-ChildItem $adoptium -Directory | Where-Object { $_.Name -like "jdk-17*" } | Sort-Object Name -Descending | Select-Object -First 1
        if ($jdk) { return $jdk.FullName }
    }
    # Fallback: derive from java.exe location
    $javaExe = Get-Command java -ErrorAction SilentlyContinue
    if ($javaExe) {
        return Split-Path (Split-Path $javaExe.Source)
    }
    return $null
}

$javaHome = Find-JavaHome
if (-not $javaHome) {
    Log "Installing Java 17..."
    choco install temurin17 -y --no-progress
    $javaHome = Find-JavaHome
    if (-not $javaHome) { Err "Java installed but JAVA_HOME could not be detected. Check C:\Program Files\Eclipse Adoptium" }
    Ok "Java 17 installed"
} else {
    Ok "Java already installed"
}
$env:JAVA_HOME = $javaHome
$env:Path = "$javaHome\bin;$env:Path"
Ok "JAVA_HOME = $env:JAVA_HOME"

# ── 3. Android SDK ────────────────────────────────────────────────────────────
$androidHome = "$env:LOCALAPPDATA\Android\Sdk"
$sdkManager  = "$androidHome\cmdline-tools\latest\bin\sdkmanager.bat"

if (-not (Test-Path "$androidHome\platform-tools")) {
    Log "Downloading Android command-line tools..."
    $cmdlineToolsZip = "$env:TEMP\cmdline-tools.zip"
    $cmdlineToolsDir = "$androidHome\cmdline-tools"
    New-Item -ItemType Directory -Force -Path $cmdlineToolsDir | Out-Null
    Invoke-WebRequest -Uri "https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip" -OutFile $cmdlineToolsZip -UseBasicParsing
    Expand-Archive -Path $cmdlineToolsZip -DestinationPath $cmdlineToolsDir -Force
    $extracted = Get-ChildItem $cmdlineToolsDir -Directory | Select-Object -First 1
    if ($extracted -and $extracted.Name -ne "latest") {
        Rename-Item $extracted.FullName "latest"
    }

    $env:ANDROID_HOME = $androidHome
    $env:Path = "$androidHome\cmdline-tools\latest\bin;$androidHome\platform-tools;$env:Path"

    Log "Accepting Android SDK licenses..."
    "y`ny`ny`ny`ny`ny`ny`ny" | & $sdkManager --licenses 2>&1 | Out-Null

    Log "Installing Android SDK packages (takes a few minutes)..."
    & $sdkManager "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;26.1.10909125"
    Ok "Android SDK installed"
} else {
    Ok "Android SDK already installed"
}

$env:ANDROID_HOME     = $androidHome
$env:ANDROID_NDK_HOME = "$androidHome\ndk\26.1.10909125"
$env:Path = "$androidHome\cmdline-tools\latest\bin;$androidHome\platform-tools;$env:Path"

# ── 4. Flutter ────────────────────────────────────────────────────────────────
$flutterDir = "C:\flutter"
if (-not (Test-Path "$flutterDir\bin\flutter.bat")) {
    Log "Downloading Flutter (this is ~700 MB, please wait)..."
    $flutterZip = "$env:TEMP\flutter.zip"
    Invoke-WebRequest -Uri "https://storage.googleapis.com/flutter_infra_release/releases/stable/windows/flutter_windows_3.24.5-stable.zip" -OutFile $flutterZip -UseBasicParsing
    Log "Extracting Flutter..."
    Expand-Archive -Path $flutterZip -DestinationPath "C:\" -Force
    Ok "Flutter installed"
} else {
    Ok "Flutter already installed"
}
$env:Path = "$flutterDir\bin;$env:Path"

# ── 5. Rust ───────────────────────────────────────────────────────────────────
$cargoBin = "$env:USERPROFILE\.cargo\bin"
$env:Path = "$cargoBin;$env:Path"

if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Log "Installing Rust..."
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing
    & $rustupInit -y --default-toolchain stable --no-modify-path
    Ok "Rust installed"
} else {
    Ok "Rust already installed"
}

# ── 6. Android Rust targets ───────────────────────────────────────────────────
Log "Adding Android Rust cross-compile targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
    cargo install cargo-ndk --locked
}
Ok "Rust Android targets ready"

# ── 7. Flutter Android config ─────────────────────────────────────────────────
Log "Configuring Flutter for Android..."
flutter config --android-sdk $androidHome 2>&1 | Out-Null
# Accept any remaining licenses non-interactively
"y`ny`ny`ny`ny`ny`ny`ny" | flutter doctor --android-licenses 2>&1 | Out-Null
Ok "Flutter configured"

# ── 8. Build APK ──────────────────────────────────────────────────────────────
$projectDir = Join-Path $PSScriptRoot "jarvis_assistant"
if (-not (Test-Path $projectDir)) {
    Err "Cannot find 'jarvis_assistant' folder. Make sure this script is in the same folder as 'jarvis_assistant'."
}

Log "Getting Flutter packages..."
Set-Location $projectDir
flutter pub get

Log "Building APK (first build can take 10-15 minutes)..."
flutter build apk --release

$apkPath = "$projectDir\build\app\outputs\flutter-apk\app-release.apk"
if (Test-Path $apkPath) {
    Write-Host ""
    Write-Host "============================================" -ForegroundColor Green
    Write-Host "  APK READY:" -ForegroundColor Green
    Write-Host "  $apkPath" -ForegroundColor White
    Write-Host "============================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "To install on your phone:" -ForegroundColor Cyan
    Write-Host "  1. Enable 'Install from unknown sources' in Android Settings > Security" -ForegroundColor White
    Write-Host "  2. Copy the APK to your phone (USB or Google Drive)" -ForegroundColor White
    Write-Host "  3. Tap the APK file on your phone to install" -ForegroundColor White
    Write-Host ""
    explorer (Split-Path $apkPath)
} else {
    Err "Build finished but APK not found. Scroll up to see what went wrong."
}

Write-Host "`nPress any key to close..." -ForegroundColor Yellow
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
