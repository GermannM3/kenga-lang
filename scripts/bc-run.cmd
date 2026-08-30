@echo off
setlocal
cd /d "%~dp0\.."

if "%~1"=="" (
  echo usage: scripts\bc-run.cmd file.kenga
  exit /b 2
)

if not exist bootstrap\bin\kenga-lite.exe (
  echo build lite first: bootstrap\build.cmd
  exit /b 1
)

echo %~1> bootstrap\generated\_bc_path.txt
bootstrap\bin\kenga-lite.exe run kenga\emit\bc_src_c.kenga
if errorlevel 1 exit /b 1

if not exist bootstrap\generated\bc_one_out.c (
  echo bc_one_out.c not generated — check _bc_path.txt
  exit /b 1
)

set VCVARS=
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
  set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
)
if "%VCVARS%"=="" if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
  set "VCVARS=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
)
if "%VCVARS%"=="" if exist "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" (
  set "VCVARS=C:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat"
)
if not "%VCVARS%"=="" (
  call "%VCVARS%" >nul
  pushd bootstrap\generated
  cl /nologo /O2 /TC bc_one_out.c /Fe:bc_one_out.exe /Fo:bc_one_out.obj /link winhttp.lib
  if errorlevel 1 (
    popd
    exit /b 1
  )
  popd
  bootstrap\generated\bc_one_out.exe
  exit /b %ERRORLEVEL%
)

where gcc >nul 2>&1
if errorlevel 1 (
  echo no compiler found ^(need MSVC vcvars or gcc^)
  exit /b 1
)

pushd bootstrap\generated
gcc -O2 -std=c99 bc_one_out.c -o bc_one_out.exe -lwinhttp
if errorlevel 1 (
  popd
  exit /b 1
)
popd
bootstrap\generated\bc_one_out.exe
exit /b %ERRORLEVEL%
