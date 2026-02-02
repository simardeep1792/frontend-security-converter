#!/bin/bash
# Deployment script for frontend-security-converter to GCP VM
# Run this script from your local machine with gcloud configured

set -e

PROJECT="sandbox-caf-compute-hub"
ZONE="northamerica-northeast1-a"
VM_NAME="security-converter-vm"
REMOTE_DIR="/opt/frontend-security-converter"
LOCAL_DIR="$(dirname "$(dirname "$(realpath "$0")")")"

echo "=== Frontend Security Converter Deployment ==="
echo "Project: $PROJECT"
echo "Zone: $ZONE"
echo "VM: $VM_NAME"
echo "Local directory: $LOCAL_DIR"
echo ""

# Create a tarball of the source code (excluding target directory and .git)
echo "Creating source tarball..."
cd "$LOCAL_DIR"
tar --exclude='target' --exclude='.git' --exclude='deploy/*.tar.gz' -czvf deploy/frontend-source.tar.gz .

# Copy the tarball to the VM using IAP tunnel
echo "Copying source to VM via IAP tunnel..."
gcloud compute scp deploy/frontend-source.tar.gz "$VM_NAME":/tmp/frontend-source.tar.gz \
    --project="$PROJECT" \
    --zone="$ZONE" \
    --tunnel-through-iap

# Copy the setup script
echo "Copying setup script to VM..."
gcloud compute scp deploy/setup-on-vm.sh "$VM_NAME":/tmp/setup-on-vm.sh \
    --project="$PROJECT" \
    --zone="$ZONE" \
    --tunnel-through-iap

# Run the setup script on the VM
echo "Running setup script on VM..."
gcloud compute ssh "$VM_NAME" \
    --project="$PROJECT" \
    --zone="$ZONE" \
    --tunnel-through-iap \
    --command="chmod +x /tmp/setup-on-vm.sh && sudo /tmp/setup-on-vm.sh"

echo ""
echo "=== Deployment Complete ==="
echo "The frontend should now be running on the VM."
echo "To check status: gcloud compute ssh $VM_NAME --project=$PROJECT --zone=$ZONE --tunnel-through-iap --command='sudo systemctl status frontend-security-converter'"
