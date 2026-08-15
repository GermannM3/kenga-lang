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

echo === kenga/emit/lower_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_c.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_kval.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/lower_kv.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\lower_kv.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/opcodes_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\opcodes_c.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/bc_vm_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\bc_vm_c.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/bc_compile_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\bc_compile_c.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/bc_src_c.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\bc_src_c.kenga
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
  for %%f in (lower_fact lower_for lower_if lower_fn lower_list lower_str lower_events lower_agent lower_import lower_for_lite lower_lists_lite lower_struct_lite lower_elif_lite lower_float_lite) do (
    cl /nologo /O2 /TC bootstrap\bin\%%f.c /Fe:bootstrap\bin\%%f.exe /Fo:bootstrap\bin\%%f.obj
    if errorlevel 1 exit /b 1
    bootstrap\bin\%%f.exe
    if errorlevel 1 exit /b 1
  )
  pushd bootstrap\generated
  for %%f in (lower_str_kv lower_events_kv lower_agent_kv lower_lex_frag lower_parse_frag bc_vm_seed bc_while_sum bc_from_src bc_from_for bc_from_fn bc_from_lists bc_from_for_list bc_from_break bc_from_for_lite) do (
    cl /nologo /O2 /TC %%f.c /Fe:%%f.exe /Fo:%%f.obj
    if errorlevel 1 exit /b 1
    %%f.exe
    if errorlevel 1 exit /b 1
  )
  popd
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
    for %%f in (lower_fact lower_for lower_if lower_fn lower_list lower_str lower_events lower_agent lower_import lower_for_lite lower_lists_lite lower_struct_lite lower_elif_lite lower_float_lite) do (
      gcc -O2 -std=c99 bootstrap\bin\%%f.c -o bootstrap\bin\%%f.exe
      if errorlevel 1 exit /b 1
      bootstrap\bin\%%f.exe
      if errorlevel 1 exit /b 1
    )
    pushd bootstrap\generated
    for %%f in (lower_str_kv lower_events_kv lower_agent_kv lower_lex_frag lower_parse_frag bc_vm_seed bc_while_sum bc_from_src bc_from_for bc_from_fn bc_from_lists bc_from_for_list bc_from_break bc_from_for_lite) do (
      gcc -O2 -std=c99 %%f.c -o %%f.exe
      if errorlevel 1 exit /b 1
      %%f.exe
      if errorlevel 1 exit /b 1
    )
    popd
  )
)

echo.
echo OK: freedom smoke ^(more + lower_c + KVal lower_kv → native^)
