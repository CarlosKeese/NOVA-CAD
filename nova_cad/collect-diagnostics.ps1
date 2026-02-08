# Script para coletar diagnósticos do Nova CAD
# Execute este script após testar a aplicação

$logPath = "$env:LOCALAPPDATA\NovaCAD\viewport_logs.txt"
$outputFile = "nova-cad-diagnostics.txt"

Write-Host "=== Nova CAD Diagnostic Collector ===" -ForegroundColor Cyan
Write-Host ""

# Check if log exists
if (Test-Path $logPath) {
    Write-Host "✓ Log file found" -ForegroundColor Green
    
    # Get last 50 lines
    $lastLines = Get-Content $logPath -Tail 50
    
    Write-Host ""
    Write-Host "=== LAST 50 LOG LINES ===" -ForegroundColor Yellow
    $lastLines | ForEach-Object { Write-Host $_ }
    
    # Save to output file
    $lastLines | Out-File $outputFile
    Write-Host ""
    Write-Host "✓ Diagnostics saved to: $outputFile" -ForegroundColor Green
} else {
    Write-Host "✗ Log file not found at: $logPath" -ForegroundColor Red
    Write-Host "  Please run Nova CAD first!" -ForegroundColor Yellow
}

# System information
Write-Host ""
Write-Host "=== SYSTEM INFORMATION ===" -ForegroundColor Yellow
Write-Host "OS: $([System.Environment]::OSVersion.VersionString)"
Write-Host "Machine: $env:COMPUTERNAME"
Write-Host "User: $env:USERNAME"

# GPU Information
Write-Host ""
Write-Host "=== GPU INFORMATION ===" -ForegroundColor Yellow
try {
    $gpu = Get-WmiObject Win32_VideoController | Select-Object -First 1
    Write-Host "GPU: $($gpu.Name)"
    Write-Host "Driver: $($gpu.DriverVersion)"
    Write-Host "Resolution: $($gpu.CurrentHorizontalResolution)x$($gpu.CurrentVerticalResolution)"
} catch {
    Write-Host "Could not retrieve GPU information" -ForegroundColor Red
}

# .NET Version
Write-Host ""
Write-Host "=== .NET VERSION ===" -ForegroundColor Yellow
dotnet --version

Write-Host ""
Write-Host "=== INSTRUCTIONS ===" -ForegroundColor Cyan
Write-Host "1. Please send the file '$outputFile' to the developer"
Write-Host "2. Or copy-paste the content above"
Write-Host "3. Also answer the questions in DIAGNOSTIC_GUIDE.md"
Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
