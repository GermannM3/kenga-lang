@echo off
setlocal
cd /d "%~dp0.."
python book\make_cover.py
if errorlevel 1 exit /b 1
python book\make_book.py
if errorlevel 1 exit /b 1
echo book\kenga_kniga_yantaras_v1.pdf
echo book\kenga_kniga_yantaras_v1.epub
