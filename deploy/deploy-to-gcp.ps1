# PowerShell Deployment script for frontend-security-converter to GCP VM
# Run this script from your local machine with gcloud configured

$ErrorActionPreference = "Stop"

$PROJECT = "sandbox-caf-compute-hub"
$ZONE = "northamerica-northeast1-a"
$VM_NAME = "security-converter-vm"
$SCRIPT_DIR = Split-Path -Parent $MyInvocation.MyCommand.Path
$LOCAL_DIR = Split-Path -Parent $SCRIPT_DIR

Write-Host "=== Frontend Security Converter Deployment ===" -ForegroundColor Cyan
Write-Host "Project: $PROJECT"
Write-Host "Zone: $ZONE"
Write-Host "VM: $VM_NAME"
Write-Host "Local directory: $LOCAL_DIR"
Write-Host ""

# Create a list of files to include (excluding target and .git)
Write-Host "Creating source tarball..." -ForegroundColor Yellow
Set-Location $LOCAL_DIR

# Use tar to create archive (available in Windows 10+)
$tarFile = Join-Path $SCRIPT_DIR "frontend-source.tar.gz"
if (Test-Path $tarFile) {
    Remove-Item $tarFile
}

# Create tar excluding target and .git directories
tar --exclude='target' --exclude='.git' --exclude='deploy/*.tar.gz' -czvf $tarFile -C $LOCAL_DIR .

if (-not (Test-Path $tarFile)) {
    Write-Host "ERROR: Failed to create tarball" -ForegroundColor Red
    exit 1
}

Write-Host "Tarball created: $tarFile" -ForegroundColor Green

# Copy the tarball to the VM using IAP tunnel
Write-Host "`nCopying source to VM via IAP tunnel..." -ForegroundColor Yellow
gcloud compute scp $tarFile "${VM_NAME}:/tmp/frontend-source.tar.gz" `
    --project="$PROJECT" `
    --zone="$ZONE" `
    --tunnel-through-iap

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to copy source to VM" -ForegroundColor Red
    exit 1
}

# Copy the .env file
Write-Host "`nCopying .env file to VM..." -ForegroundColor Yellow
$envFile = Join-Path $LOCAL_DIR ".env"
if (Test-Path $envFile) {
    gcloud compute scp $envFile "${VM_NAME}:/tmp/.env" `
        --project="$PROJECT" `
        --zone="$ZONE" `
        --tunnel-through-iap

    if ($LASTEXITCODE -ne 0) {
        Write-Host "WARNING: Failed to copy .env file, will use defaults" -ForegroundColor Yellow
    } else {
        Write-Host ".env file copied successfully" -ForegroundColor Green
    }
} else {
    Write-Host "WARNING: No .env file found, will use defaults" -ForegroundColor Yellow
}

# Copy the setup script
Write-Host "`nCopying setup script to VM..." -ForegroundColor Yellow
$setupScript = Join-Path $SCRIPT_DIR "setup-on-vm.sh"
gcloud compute scp $setupScript "${VM_NAME}:/tmp/setup-on-vm.sh" `
    --project="$PROJECT" `
    --zone="$ZONE" `
    --tunnel-through-iap

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to copy setup script to VM" -ForegroundColor Red
    exit 1
}

# Run the setup script on the VM
Write-Host "`nRunning setup script on VM (this may take several minutes for build)..." -ForegroundColor Yellow
gcloud compute ssh $VM_NAME `
    --project="$PROJECT" `
    --zone="$ZONE" `
    --tunnel-through-iap `
    --command="chmod +x /tmp/setup-on-vm.sh && sudo /tmp/setup-on-vm.sh"

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Setup script failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=== Deployment Complete ===" -ForegroundColor Green
Write-Host "The frontend should now be running on the VM."
Write-Host ""
Write-Host "Useful commands:" -ForegroundColor Cyan
Write-Host "  Check status:"
Write-Host "    gcloud compute ssh $VM_NAME --project=$PROJECT --zone=$ZONE --tunnel-through-iap --command='sudo systemctl status frontend-security-converter'"
Write-Host ""
Write-Host "  View logs:"
Write-Host "    gcloud compute ssh $VM_NAME --project=$PROJECT --zone=$ZONE --tunnel-through-iap --command='sudo journalctl -u frontend-security-converter -f'"
Write-Host ""
Write-Host "  SSH into VM:"
Write-Host "    gcloud compute ssh $VM_NAME --project=$PROJECT --zone=$ZONE --tunnel-through-iap"
