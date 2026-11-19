#!/usr/bin/env bash
set -euo pipefail

package=br.com.margem.app
adb install target/installer/Margem-android.apk
adb shell wm size 480x900
adb shell wm density 160
adb shell am start -W -n "$package/com.google.androidgamesdk.GameActivity"
sleep 3
if ! adb shell pidof "$package"; then
    adb logcat -d -s AndroidRuntime RustStdoutStderr
    exit 1
fi
adb shell input tap 436 42  # Nova página, abaixo da barra de status
sleep 1
adb shell input text MargemAndroid
sleep 1
adb shell run-as "$package" cat files/workspace.json | grep -q '"title": "MargemAndroid"'
