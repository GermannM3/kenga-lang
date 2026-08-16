@echo off
setlocal
cd /d "%~dp0\.."

if not exist bootstrap\bin\kenga-lite.exe (
  echo build lite first: bootstrap\build.cmd
  exit /b 1
)

echo === birth: model writes examples\ml\kenga_born.kenga ===
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_birth.kenga
if errorlevel 1 exit /b 1

echo.
echo === born: run what the model wrote ===
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_born.kenga
if errorlevel 1 exit /b 1

echo.
echo.
echo native C (same birth, no lite VM): scripts\bc-run.cmd examples\ml\kenga_birth.kenga
echo OK: Kenga wrote a program and ran it
