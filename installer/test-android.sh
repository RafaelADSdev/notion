#!/usr/bin/env bash
set -euo pipefail

package=br.com.margem.app
mkdir -p target/android-smoke
trap 'adb exec-out screencap -p > target/android-smoke/final.png || true
    adb logcat -d -s AndroidRuntime RustStdoutStderr Margem GameActivity android-activity > target/android-smoke/runtime.log || true' EXIT
adb install target/installer/Margem-android.apk
adb shell wm size 480x900
adb shell wm density 160
adb shell input keyevent KEYCODE_WAKEUP
adb shell wm dismiss-keyguard
activity=$(adb shell cmd package resolve-activity --brief "$package" | tail -1 | tr -d '\r')
adb shell am start -W -n "$activity"
sleep 9
adb exec-out screencap -p > target/android-smoke/inicio.png
if ! adb shell pidof "$package"; then
    adb logcat -d -s AndroidRuntime RustStdoutStderr
    exit 1
fi
workspace=files/workspace.json
for y in 221 197 239; do
    if adb shell run-as "$package" test -f "$workspace"; then break; fi
    adb shell input tap 65 "$y"
    sleep 1
done
adb shell run-as "$package" test -f "$workspace"
adb shell input text MargemAndroid
sleep 1
adb shell run-as "$package" cat "$workspace" | grep -q '"title": "MargemAndroid"'
