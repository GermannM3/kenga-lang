@echo off
setlocal
cd /d "%~dp0\.."

if not exist bootstrap\bin\kenga-lite.exe (
  echo build lite first: bootstrap\build.cmd
  exit /b 1
)

echo === kenga/compiler/more.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\compiler\more.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/c_seed.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\c_seed.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/expr_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\expr_c.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/control_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\control_c.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/core_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\core_c.kenga
if errorlevel 1 exit /b 1

set VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
if exist "%VCVARS%" (
  call "%VCVARS%" >nul
  cl /nologo /O2 /TC bootstrap\bin\expr_from_kenga.c /Fe:bootstrap\bin\expr_from_kenga.exe /Fo:bootstrap\bin\expr_from_kenga.obj
  if errorlevel 1 exit /b 1
  bootstrap\bin\expr_from_kenga.exe
  if errorlevel 1 exit /b 1
  cl /nologo /O2 /TC bootstrap\bin\control_from_kenga.c /Fe:bootstrap\bin\control_from_kenga.exe /Fo:bootstrap\bin\control_from_kenga.obj
  if errorlevel 1 exit /b 1
  bootstrap\bin\control_from_kenga.exe
  if errorlevel 1 exit /b 1
  cl /nologo /O2 /TC bootstrap\bin\mini_from_kenga.c /Fe:bootstrap\bin\mini_from_kenga.exe /Fo:bootstrap\bin\mini_from_kenga.obj
  if errorlevel 1 exit /b 1
  bootstrap\bin\mini_from_kenga.exe
  if errorlevel 1 exit /b 1
  cl /nologo /O2 /TC bootstrap\bin\core_from_kenga.c /Fe:bootstrap\bin\core_from_kenga.exe /Fo:bootstrap\bin\core_from_kenga.obj
  if errorlevel 1 exit /b 1
  bootstrap\bin\core_from_kenga.exe
  if errorlevel 1 exit /b 1
) else (
  where gcc >nul 2>&1 && (
    gcc -O2 -std=c99 bootstrap\bin\expr_from_kenga.c -o bootstrap\bin\expr_from_kenga.exe
    if errorlevel 1 exit /b 1
    bootstrap\bin\expr_from_kenga.exe
    if errorlevel 1 exit /b 1
    gcc -O2 -std=c99 bootstrap\bin\control_from_kenga.c -o bootstrap\bin\control_from_kenga.exe
    if errorlevel 1 exit /b 1
    bootstrap\bin\control_from_kenga.exe
    if errorlevel 1 exit /b 1
    gcc -O2 -std=c99 bootstrap\bin\mini_from_kenga.c -o bootstrap\bin\mini_from_kenga.exe
    if errorlevel 1 exit /b 1
    bootstrap\bin\mini_from_kenga.exe
    if errorlevel 1 exit /b 1
    gcc -O2 -std=c99 bootstrap\bin\core_from_kenga.c -o bootstrap\bin\core_from_kenga.exe
    if errorlevel 1 exit /b 1
    bootstrap\bin\core_from_kenga.exe
    if errorlevel 1 exit /b 1
  )
)

echo.
echo OK: freedom smoke ^(more dialect + Kenga emit → C → native^)
