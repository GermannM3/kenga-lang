@echo off
setlocal
cd /d "%~dp0"
set VSIX=%~dp0vscode\kenga-3.10.0.vsix

if not exist "%VSIX%" (
  where npx >nul 2>&1 || (
    echo Need Node.js/npx to build the VSIX, or ship kenga-3.10.0.vsix in editors\vscode\
    exit /b 1
  )
  pushd vscode
  call npx --yes @vscode/vsce package --skip-license -o kenga-3.10.0.vsix
  if errorlevel 1 exit /b 1
  popd
)

where cursor >nul 2>&1 && (
  cursor --install-extension "%VSIX%"
  goto :icons
)
where code >nul 2>&1 && (
  code --install-extension "%VSIX%"
  goto :icons
)
echo Neither cursor nor code found in PATH.
echo VSIX is at %VSIX%
exit /b 1

:icons
rem Material Icon Theme custom SVG (Cursor + VS Code)
if not exist "%USERPROFILE%\.cursor\extensions\icons" mkdir "%USERPROFILE%\.cursor\extensions\icons"
copy /Y "%~dp0vscode\icons\kenga-file.svg" "%USERPROFILE%\.cursor\extensions\icons\kenga.svg" >nul
if not exist "%USERPROFILE%\.vscode\extensions\icons" mkdir "%USERPROFILE%\.vscode\extensions\icons"
copy /Y "%~dp0vscode\icons\kenga-file.svg" "%USERPROFILE%\.vscode\extensions\icons\kenga.svg" >nul 2>nul

echo.
echo OK. Reload Window.
echo If .kenga still looks like plain text ^(Material Icon Theme^), add to User Settings:
echo   "material-icon-theme.files.associations": { "*.kenga": "../../icons/kenga" }
echo.
