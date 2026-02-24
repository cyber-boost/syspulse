param(
    [string]$PythonVersion = "3.12",
    [string]$VenvPath = ".venv312",
    [switch]$Workspace,
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command py -ErrorAction SilentlyContinue)) {
    throw "Python launcher 'py' was not found in PATH. Install Python from python.org first."
}

if (-not (Test-Path $VenvPath)) {
    Write-Host "Creating virtual environment at '$VenvPath' with Python $PythonVersion..."
    py -$PythonVersion -m venv $VenvPath
}

$pythonPath = Join-Path $VenvPath "Scripts/python.exe"
if (-not (Test-Path $pythonPath)) {
    throw "Virtual environment python not found at '$pythonPath'."
}

$resolvedPython = (Resolve-Path $pythonPath).Path
$env:PYO3_PYTHON = $resolvedPython
Write-Host "Using PYO3_PYTHON=$resolvedPython"

if ($NoBuild) {
    Write-Host "Skipping cargo build because -NoBuild was provided."
    return
}

if ($Workspace) {
    Write-Host "Running: cargo build"
    cargo build
} else {
    Write-Host "Running: cargo build -p syspulse-py"
    cargo build -p syspulse-py
}
