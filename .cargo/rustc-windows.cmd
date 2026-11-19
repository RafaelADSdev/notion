@echo off
setlocal
set "RUSTUP_HOME=%~dp0..\target\rustup"
rustup toolchain install 1.97.1-x86_64-pc-windows-gnullvm --profile minimal --component rust-mingw --no-self-update
if errorlevel 1 exit /b 1
rustup toolchain install 1.97.1-x86_64-pc-windows-gnu --profile minimal --component rust-mingw --no-self-update
exit /b %errorlevel%
