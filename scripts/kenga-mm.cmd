@echo off
setlocal
cd /d "%~dp0\.."

if not exist bootstrap\bin\kenga-lite.exe (
  echo build lite first: bootstrap\build.cmd
  exit /b 1
)

if not exist minds\kenga_mm_w.kt (
  echo === train mm seed ===
  bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_lm.kenga
  if errorlevel 1 exit /b 1
)

echo === talk: caption from saved weights ===
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_talk.kenga
if errorlevel 1 exit /b 1

echo === decoder: next-char from the same scenes ===
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_gen.kenga
if errorlevel 1 exit /b 1

echo === word decoder: full color token ===
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_mm_words.kenga
if errorlevel 1 exit /b 1

echo OK: PPM+WAV → text (linear + char stem + word caption)
