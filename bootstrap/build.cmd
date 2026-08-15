@echo off
setlocal
cd /d "%~dp0"
if not exist bin mkdir bin

set VCVARS=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
if exist "%VCVARS%" (
  call "%VCVARS%" >nul
  cl /nologo /O2 /TC kenga_lite.c /Fe:bin\kenga-lite.exe /Fo:bin\kenga_lite.obj
  if errorlevel 1 exit /b 1
  goto :run
)

where gcc >nul 2>&1 && (
  gcc -O2 -std=c99 kenga_lite.c -o bin\kenga-lite.exe
  if errorlevel 1 exit /b 1
  goto :run
)

where clang >nul 2>&1 && (
  clang -O2 -std=c99 kenga_lite.c -o bin\kenga-lite.exe
  if errorlevel 1 exit /b 1
  goto :run
)

echo No C compiler found (need MSVC cl, gcc, or clang with CRT).
exit /b 1

:run
echo.
bin\kenga-lite.exe
if errorlevel 1 exit /b 1
echo.
echo Try: bootstrap\bin\kenga-lite.exe run examples\selfhost\fact_lite.kenga
