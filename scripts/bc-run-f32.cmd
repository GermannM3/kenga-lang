@echo off
setlocal
cd /d "%~dp0\.."

if not exist bootstrap\generated\bc_one_out.c (
  echo generate first: scripts\bc-run.cmd file.kenga
  exit /b 1
)

set VCVARS=
if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
  set "VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
)
if "%VCVARS%"=="" if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
  set "VCVARS=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
)
if not "%VCVARS%"=="" (
  call "%VCVARS%" >nul
  pushd bootstrap\generated
  cl /nologo /O2 /TC /DKENGA_TENSOR_F32 bc_one_out.c /Fe:bc_f32.exe /Fo:bc_f32.obj
  if errorlevel 1 ( popd & exit /b 1 )
  popd
  bootstrap\generated\bc_f32.exe
  exit /b %ERRORLEVEL%
)

where gcc >nul 2>&1
if errorlevel 1 (
  echo no compiler found ^(need MSVC vcvars or gcc^)
  exit /b 1
)
pushd bootstrap\generated
gcc -O2 -std=c99 -DKENGA_TENSOR_F32 bc_one_out.c -o bc_f32.exe
if errorlevel 1 ( popd & exit /b 1 )
popd
bootstrap\generated\bc_f32.exe
exit /b %ERRORLEVEL%
