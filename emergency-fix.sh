#!/bin/bash
# Emergency fix for stuck system

echo "=== Emergency System Recovery ==="
echo
echo "⚠️  WARNING: Your system has 61 processes in D-state (uninterruptible sleep)"
echo "This is preventing swap from being cleared and causing system instability."
echo

echo "Immediate actions you can take:"
echo
echo "1. Kill the firmware update daemon (may help with D-state processes):"
echo "   sudo systemctl stop fwupd.service"
echo "   sudo systemctl disable fwupd.service"
echo

echo "2. Try to reduce KDE memory usage:"
echo "   killall plasmashell && kstart5 plasmashell"
echo

echo "3. Force drop all caches:"
echo "   sudo sync && echo 3 | sudo tee /proc/sys/vm/drop_caches"
echo

echo "4. If above doesn't work, the ONLY solution is to reboot:"
echo "   sudo reboot"
echo

echo "=== Root Cause ==="
echo "The fwupd (firmware update daemon) appears to be stuck in a loop,"
echo "creating many D-state processes that cannot be killed."
echo "This is likely a driver or hardware issue."
echo

echo "=== Prevention ==="
echo "After reboot:"
echo "1. Disable fwupd if not needed: sudo systemctl disable fwupd"
echo "2. Increase swap size to prevent full swap: sudo fallocate -l 4G /swapfile2"
echo "3. Set swappiness lower: echo 'vm.swappiness=10' | sudo tee -a /etc/sysctl.conf"
echo "4. Monitor with: watch -n 1 'ps aux | grep -c " D "'  # Should be 0-2, not 61"