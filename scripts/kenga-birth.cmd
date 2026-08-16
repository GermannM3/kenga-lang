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

if not exist bootstrap\generated\bc_from_birth.c goto :done
if "%VCVARS%"=="" goto :trygcc

echo.
echo === native C birth (bytecode VM, no lite) ===
call "%VCVARS%" >nul
cl /nologo /O2 /TC bootstrap\generated\bc_from_birth.c /Fe:bootstrap\generated\bc_from_birth.exe /Fo:bootstrap\generated\bc_from_birth.obj
if errorlevel 1 exit /b 1
bootstrap\generated\bc_from_birth.exe
if errorlevel 1 exit /b 1
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_born.kenga
if errorlevel 1 exit /b 1
goto :done

:trygcc
where gcc >nul 2>&1
if errorlevel 1 goto :done
echo.
echo === native C birth (bytecode VM, no lite) ===
gcc -O2 -std=c99 bootstrap\generated\bc_from_birth.c -o bootstrap\generated\bc_from_birth.exe
if errorlevel 1 exit /b 1
bootstrap\generated\bc_from_birth.exe
if errorlevel 1 exit /b 1
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_born.kenga
if errorlevel 1 exit /b 1

:done
echo.
echo OK: Kenga wrote a program and ran it
