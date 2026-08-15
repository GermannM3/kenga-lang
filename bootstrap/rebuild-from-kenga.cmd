@echo off
setlocal
cd /d "%~dp0\.."

echo === freedom path first (no cargo): scripts\freedom-smoke.cmd ===
call scripts\freedom-smoke.cmd
if errorlevel 1 exit /b 1

echo.
echo === optional: full lite chicken-egg via Rust emit-c (legacy until emit covers dialect) ===
cargo run --quiet -- emit-c examples/selfhost/kenga_lite.kenga -o bootstrap\kenga_lite.gen.c
if errorlevel 1 (
  echo skip: cargo emit-c not available
  exit /b 0
)

set VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
if not exist bin mkdir bootstrap\bin 2>nul
if not exist bootstrap\bin mkdir bootstrap\bin

if exist "%VCVARS%" (
  call "%VCVARS%" >nul
  cl /nologo /O2 /TC bootstrap\kenga_lite.gen.c /Fe:bootstrap\bin\kenga-lite-gen.exe /Fo:bootstrap\bin\kenga_lite_gen.obj
  if errorlevel 1 exit /b 1
) else (
  where gcc >nul 2>&1 && gcc -O2 -std=c99 bootstrap\kenga_lite.gen.c -o bootstrap\bin\kenga-lite-gen.exe
  if errorlevel 1 exit /b 1
)

echo.
echo === run generated lite (from .kenga via emit-c) ===
bootstrap\bin\kenga-lite-gen.exe
if errorlevel 1 exit /b 1
echo.
echo OK: chicken-egg path for lite works
