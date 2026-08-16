@echo off
setlocal
cd /d "%~dp0\.."

set OUT=dist\hf-kenga-seed
if exist "%OUT%" rmdir /s /q "%OUT%"
mkdir "%OUT%"
mkdir "%OUT%\examples"
mkdir "%OUT%\examples\ml"
mkdir "%OUT%\examples\ml\assets"
mkdir "%OUT%\minds"

copy /y hf\kenga-seed\README.md "%OUT%\README.md" >nul
if exist hf\kenga-seed\weights.md copy /y hf\kenga-seed\weights.md "%OUT%\weights.md" >nul
copy /y docs\HUGGINGFACE.md "%OUT%\HUGGINGFACE.md" >nul
copy /y docs\KENGA_LM.md "%OUT%\KENGA_LM.md" >nul

copy /y examples\ml\kenga_mm_core.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_mm_lm.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_mm_talk.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_mm_gen.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_mm_gen_talk.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_mm_words.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_birth.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_born.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_dec.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_charlm.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\kenga_seed.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\living_multimodal.kenga "%OUT%\examples\ml\" >nul
copy /y examples\ml\assets\frame0.ppm "%OUT%\examples\ml\assets\" >nul
copy /y examples\ml\assets\frame1.ppm "%OUT%\examples\ml\assets\" >nul
copy /y examples\ml\assets\frame2.ppm "%OUT%\examples\ml\assets\" >nul
copy /y examples\ml\assets\tone0.wav "%OUT%\examples\ml\assets\" >nul
copy /y examples\ml\assets\tone1.wav "%OUT%\examples\ml\assets\" >nul
copy /y examples\ml\assets\tone2.wav "%OUT%\examples\ml\assets\" >nul

if exist minds\kenga_mm_w.kt copy /y minds\kenga_mm_w.kt "%OUT%\minds\" >nul
if exist minds\kenga_mm_b.kt copy /y minds\kenga_mm_b.kt "%OUT%\minds\" >nul
if exist minds\multi.km copy /y minds\multi.km "%OUT%\minds\" >nul

echo packed %OUT%
echo next: huggingface-cli upload Kenga-ai/kenga-seed-mm dist\hf-kenga-seed --repo-type model
