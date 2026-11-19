@echo off
setlocal
set "app=%~dp0target\debug\margem-desktop.exe"
if not exist "%app%" (
    echo O Margem precisa ser compilado no Craft Anything antes de abrir.
    pause
    exit /b 1
)
start "" /D "%~dp0." "%app%"
