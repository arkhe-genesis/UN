# Cathedral OS Makefile

.PHONY: all linux-iso android-rom windows-msi

all:
	@echo "Use 'make linux-iso', 'make android-rom', or 'make windows-msi'"

linux-iso:
	cd platforms/linux && ./build_live_usb.sh

android-rom:
	@echo "Android ROM build requires AOSP root. See platforms/android/build_rom.sh"

windows-msi:
	@echo "Windows MSI build requires PowerShell and WiX. See platforms/windows/build_msi.ps1"
