#!/bin/bash

# Exit on error
set -e

echo "Installing PenumbraOS. Builds from 2025-08-06"

cd $(mktemp -d)

# ----------------

echo "Setting up pinitd"
curl -L -O https://github.com/PenumbraOS/pinitd/releases/download/2025-08-06.1/pinitd-cli
curl -L -O https://github.com/PenumbraOS/pinitd/releases/download/2025-08-06.1/pinitd.apk

adb install pinitd.apk
adb shell mkdir -p /data/local/tmp/bin/
adb push pinitd-cli /data/local/tmp/bin/pinitd-cli
adb shell chmod +x /data/local/tmp/bin/pinitd-cli

adb shell mkdir -p /sdcard/penumbra/etc/pinitd/system/enabled/

adb shell pm grant com.penumbraos.pinitd android.permission.WRITE_SECURE_SETTINGS
adb shell pm grant com.penumbraos.pinitd android.permission.READ_LOGS
adb shell appops set com.penumbraos.pinitd MANAGE_EXTERNAL_STORAGE allow

# ----------------

echo "Setting up PenumbraOS SDK"
curl -L -O https://github.com/PenumbraOS/sdk/releases/download/2025-08-06.0/SDK-Bridge-Core.apk
curl -L -O https://github.com/PenumbraOS/sdk/releases/download/2025-08-06.0/SDK-Bridge-Settings.apk
curl -L -O https://github.com/PenumbraOS/sdk/releases/download/2025-08-06.0/SDK-Bridge-Shell.apk
curl -L -O https://github.com/PenumbraOS/sdk/releases/download/2025-08-06.0/SDK-Bridge-System.apk
curl -L -O https://github.com/PenumbraOS/sdk/raw/3084d5f11eda79453f5c778ad7ca067a40a807da/config/pinitd/bridge_service.unit
curl -L -O https://github.com/PenumbraOS/sdk/raw/3084d5f11eda79453f5c778ad7ca067a40a807da/config/pinitd/bridge_settings.unit
curl -L -O https://github.com/PenumbraOS/sdk/raw/3084d5f11eda79453f5c778ad7ca067a40a807da/config/pinitd/bridge_shell_service.unit
curl -L -O https://github.com/PenumbraOS/sdk/raw/3084d5f11eda79453f5c778ad7ca067a40a807da/config/pinitd/bridge_system_service.unit

adb install SDK-Bridge-Core.apk
adb install SDK-Bridge-Settings.apk
adb install SDK-Bridge-Shell.apk
adb install SDK-Bridge-System.apk

adb push bridge_service.unit /sdcard/penumbra/etc/pinitd/system/
adb push bridge_settings.unit /sdcard/penumbra/etc/pinitd/system/
adb push bridge_shell_service.unit /sdcard/penumbra/etc/pinitd/system/
adb push bridge_system_service.unit /sdcard/penumbra/etc/pinitd/system/

adb shell touch /sdcard/penumbra/etc/pinitd/system/enabled/bridge_service.unit
adb shell touch /sdcard/penumbra/etc/pinitd/system/enabled/bridge_settings.unit
adb shell touch /sdcard/penumbra/etc/pinitd/system/enabled/bridge_shell_service.unit
adb shell touch /sdcard/penumbra/etc/pinitd/system/enabled/bridge_system_service.unit

# ----------------

echo "Setting up MABL"
curl -L -O https://github.com/PenumbraOS/mabl/releases/download/2025-08-06.0/MABL-AiPin.apk
curl -L -O https://github.com/PenumbraOS/mabl/releases/download/2025-08-06.0/Plugin-AiPin-System.apk
curl -L -O https://github.com/PenumbraOS/mabl/releases/download/2025-08-06.0/Plugin-Demo.apk
curl -L -O https://github.com/PenumbraOS/mabl/releases/download/2025-08-06.0/Plugin-Generic-System.apk
curl -L -O https://github.com/PenumbraOS/mabl/releases/download/2025-08-06.0/Plugin-OpenAI.apk

adb install MABL-AiPin.apk
adb install Plugin-AiPin-System.apk
adb install Plugin-Demo.apk
adb install Plugin-Generic-System.apk
adb install Plugin-OpenAI.apk

adb shell appops set com.penumbraos.plugins.openai MANAGE_EXTERNAL_STORAGE allow
adb shell pm disable-user --user 0 humane.experience.systemnavigation
sleep 1
adb shell cmd package set-home-activity com.penumbraos.mabl/.MainActivity
sleep 1
# I think one of these works
adb shell settings put secure launcher_component com.penumbraos.mabl/.MainActivity
adb shell settings put system home_app com.penumbraos.mabl
adb shell settings put global default_launcher com.penumbraos.mabl/.MainActivity

# ----------------

# TODO: Should probably delete temp dir

echo "PenumbraOS installed. Reboot your device to boot into MABL (this may take several minutes)"