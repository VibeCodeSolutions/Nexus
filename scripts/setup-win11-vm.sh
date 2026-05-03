#!/usr/bin/env bash
# scripts/setup-win11-vm.sh — Win11 Enterprise Eval VM in VirtualBox für NEXUS-Debug.
# Richtet nexus-win11-eval (8 GB RAM, 4 vCPUs, 80 GB VDI, EFI, TPM 2.0) ein.
# ISO-Bezug: https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise
#
# Usage: scripts/setup-win11-vm.sh [--reset] /pfad/zum/win11.iso
#        WIN11_ISO_PATH=/pfad/zum/win11.iso scripts/setup-win11-vm.sh
#
# Nach erfolgreichem Lauf:
#   VBoxManage list vms  → zeigt "nexus-win11-eval"
#   VirtualBox-GUI öffnen → VM starten → Setup-Wizard durchklicken
#   NEXUS-MSI via Shared Folder in VM installieren
#   DevTools: Rechtsklick in der App → Inspect Element
#
# 90-Tage-Eval-Reset: scripts/setup-win11-vm.sh --reset /pfad/zum/win11.iso
#   Löscht VM + Disk und legt alles neu an.
#
# Plan B (KVM/QEMU falls VirtualBox-Kernel-Modul nicht läuft):
#   sudo dnf install virt-manager qemu-kvm libvirt swtpm ovmf
#   qemu-img create -f qcow2 ~/nexus-win11.qcow2 80G
#   virt-install --name nexus-win11-eval --ram 8192 --vcpus 4 \
#     --disk path=~/nexus-win11.qcow2 --cdrom /pfad/win11.iso \
#     --os-variant win11 --boot uefi --features smm=on --machine q35 \
#     --tpm model=tpm-crb,backend.type=emulator,backend.version=2.0

set -euo pipefail

# ---------------------------------------------------------------------------
# Konfiguration
# ---------------------------------------------------------------------------
VM_NAME="nexus-win11-eval"
VM_RAM_MB=8192
VM_CPUS=4
VM_VRAM_MB=256
VM_DISK_GB=80
VM_DISK_SIZE_MB=$(( VM_DISK_GB * 1024 ))
NEXUS_API_PORT=7777
MIN_FREE_GB=50

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------
log() { printf '[%s] %s\n' "$(date -u +%FT%TZ)" "$*"; }
err() { printf '[%s] ERROR: %s\n' "$(date -u +%FT%TZ)" "$*" >&2; }
die() { err "$*"; exit 1; }

# ---------------------------------------------------------------------------
# Guards
# ---------------------------------------------------------------------------
check_vboxmanage() {
  command -v VBoxManage &>/dev/null \
    || die "VBoxManage nicht gefunden. VirtualBox installieren (sudo dnf install VirtualBox)."
  local ver
  ver=$(VBoxManage --version)
  log "VirtualBox $ver gefunden."
}

check_disk_space() {
  local free_kb free_gb
  free_kb=$(df -k "${HOME}" | awk 'NR==2 {print $4}')
  free_gb=$(( free_kb / 1048576 ))
  [[ $free_gb -ge $MIN_FREE_GB ]] \
    || die "Zu wenig Speicher auf ${HOME}: ${free_gb} GB frei, ${MIN_FREE_GB} GB benötigt."
  log "Disk-Space OK: ${free_gb} GB frei."
}

get_iso_path() {
  if [[ -n "${1:-}" ]]; then
    ISO_PATH="$1"
  elif [[ -n "${WIN11_ISO_PATH:-}" ]]; then
    ISO_PATH="$WIN11_ISO_PATH"
  else
    die "ISO-Pfad fehlt. Argument übergeben oder WIN11_ISO_PATH=... setzen."
  fi
  [[ -f "$ISO_PATH" ]] || die "ISO nicht gefunden: $ISO_PATH"
  log "ISO: $ISO_PATH"
}

# ---------------------------------------------------------------------------
# VM-Funktionen
# ---------------------------------------------------------------------------
vm_exists() {
  VBoxManage list vms 2>/dev/null | grep -q "\"${VM_NAME}\""
}

delete_vm() {
  if vm_exists; then
    log "Lösche bestehende VM: $VM_NAME ..."
    VBoxManage unregistervm "$VM_NAME" --delete
    log "VM gelöscht."
  else
    log "Keine bestehende VM '$VM_NAME' — skip delete."
  fi
}

create_vm() {
  if vm_exists; then
    log "VM '$VM_NAME' existiert bereits — skip create."
    return 0
  fi
  log "Erstelle VM: $VM_NAME ..."
  VBoxManage createvm --name "$VM_NAME" --ostype Windows11_64 --register

  VBoxManage modifyvm "$VM_NAME" \
    --memory "$VM_RAM_MB" \
    --cpus "$VM_CPUS" \
    --vram "$VM_VRAM_MB" \
    --firmware efi \
    --graphicscontroller vmsvga \
    --boot1 dvd \
    --boot2 disk \
    --boot3 none \
    --boot4 none \
    --mouse usbtablet \
    --keyboard usb \
    --audio-driver none

  # TPM 2.0 — VirtualBox 7+
  VBoxManage modifyvm "$VM_NAME" --tpm-type 2.0

  # Secure Boot manuell: VM-Einstellungen → System → EFI → "Secure Boot aktivieren"
  log "Hinweis: Secure Boot manuell aktivieren → VM-Einstellungen → System → EFI."
  log "VM-Hardware konfiguriert."
}

setup_storage() {
  local disk_dir disk_path
  disk_dir="${HOME}/VirtualBox VMs/${VM_NAME}"
  disk_path="${disk_dir}/${VM_NAME}.vdi"

  # SATA Controller
  if VBoxManage showvminfo "$VM_NAME" 2>/dev/null | grep -q "SATA Controller"; then
    log "SATA Controller existiert — skip."
  else
    log "Füge SATA Controller hinzu ..."
    VBoxManage storagectl "$VM_NAME" --name "SATA Controller" \
      --add sata --controller IntelAhci --portcount 2
  fi

  # VDI-Disk
  if [[ -f "$disk_path" ]]; then
    log "VDI existiert bereits: $disk_path — skip createmedium."
  else
    log "Erstelle VDI (${VM_DISK_GB} GB dynamic) ..."
    VBoxManage createmedium disk \
      --filename "$disk_path" \
      --size "$VM_DISK_SIZE_MB" \
      --format VDI \
      --variant Standard
  fi

  # Disk einhängen
  local mr
  mr=$(VBoxManage showvminfo "$VM_NAME" --machinereadable 2>/dev/null || true)
  if printf '%s\n' "$mr" | grep -qE '^"SATA Controller-0-0"="[^"]+\.vdi"$'; then
    log "Disk bereits eingehängt — skip attach."
  else
    log "Hänge Disk ein ..."
    VBoxManage storageattach "$VM_NAME" \
      --storagectl "SATA Controller" \
      --port 0 --device 0 --type hdd \
      --medium "$disk_path"
  fi

  # IDE Controller für DVD
  if printf '%s\n' "$mr" | grep -q 'storagecontrollername[0-9]*="IDE Controller"'; then
    log "IDE Controller existiert — skip."
  else
    log "Füge IDE Controller hinzu ..."
    VBoxManage storagectl "$VM_NAME" --name "IDE Controller" --add ide
  fi
}

mount_iso() {
  local current_dvd
  current_dvd=$(VBoxManage showvminfo "$VM_NAME" --machinereadable 2>/dev/null \
    | awk -F= '/^"IDE Controller-1-0"=/ { sub(/^"/, "", $2); sub(/"$/, "", $2); print $2 }' \
    || true)

  if [[ "$current_dvd" == "$ISO_PATH" ]]; then
    log "ISO bereits eingehängt — skip."
    return 0
  fi
  log "Mounte ISO: $ISO_PATH ..."
  VBoxManage storageattach "$VM_NAME" \
    --storagectl "IDE Controller" \
    --port 1 --device 0 --type dvddrive \
    --medium "$ISO_PATH"
}

setup_network() {
  if VBoxManage showvminfo "$VM_NAME" --machinereadable 2>/dev/null \
      | grep -qE '^Forwarding\([0-9]+\)="nexus-api,'; then
    log "Port-Forward ${NEXUS_API_PORT} bereits konfiguriert — skip."
  else
    log "Konfiguriere NAT + Port-Forward ${NEXUS_API_PORT}→${NEXUS_API_PORT} ..."
    VBoxManage modifyvm "$VM_NAME" --nic1 nat
    VBoxManage modifyvm "$VM_NAME" \
      --natpf1 "nexus-api,tcp,,${NEXUS_API_PORT},,${NEXUS_API_PORT}"
  fi
}

# ---------------------------------------------------------------------------
# Hilfsfunktionen
# ---------------------------------------------------------------------------
usage() {
  cat <<EOF
Usage: $(basename "$0") [--reset] <iso-path>
       WIN11_ISO_PATH=<iso-path> $(basename "$0") [--reset]

Richtet VirtualBox-VM '$VM_NAME' für NEXUS-Win11-Debug ein.

Optionen:
  --reset     Löscht bestehende VM + Disk, legt neu an (90-Tage-Eval-Reset)
  -h, --help  Diese Hilfe

Umgebungsvariablen:
  WIN11_ISO_PATH  Pfad zur Win11-ISO (Alternative zu positivem Argument)
EOF
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
main() {
  local reset=0
  local iso_arg=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      -h|--help) usage; exit 0 ;;
      --reset)   reset=1; shift ;;
      -*)        die "Unbekannte Option: $1. -h für Hilfe." ;;
      *)         iso_arg="$1"; shift ;;
    esac
  done

  log "=== setup-win11-vm.sh — Sprint Crystalline Crab ==="
  check_vboxmanage
  check_disk_space
  get_iso_path "$iso_arg"

  if [[ $reset -eq 1 ]]; then
    log "RESET-Modus: lösche und lege neu an."
    delete_vm
  fi

  create_vm
  setup_storage
  setup_network
  mount_iso

  log "=== Fertig. Nächste Schritte: ==="
  log "  1. VirtualBox-GUI öffnen"
  log "  2. VM '$VM_NAME' starten"
  log "  3. Windows-Setup-Wizard durchlaufen (Lizenzkey überspringen ist OK bei Eval)"
  log "  4. Secure Boot: VM-Einstellungen → System → EFI → aktivieren"
  log "  5. NEXUS-MSI in VM installieren (Shared Folder empfohlen)"
  log "  6. DevTools testen: Rechtsklick in App → Inspect Element"
}

main "$@"
