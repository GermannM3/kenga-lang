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

echo === kenga/emit/rt_cli.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_cli.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_mem.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_mem.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_host.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_host.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_val.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_val.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_lex.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_lex.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_arena.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_arena.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_parse.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_parse.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_loop.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_loop.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_prog.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_prog.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_scan.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_scan.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_expr.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_expr.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_factor.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_factor.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_stmt.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_stmt.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_compile.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_compile.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_print.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_print.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_vm.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_vm.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_selftest.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_selftest.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_types.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_types.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_prophet.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_prophet.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_tensor.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_tensor.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_tape.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_tape.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_events.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_events.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_chat.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_chat.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_kval_mem.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval_mem.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_kval_tensor.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval_tensor.kenga
if errorlevel 1 exit /b 1

echo === kenga/emit/rt_kval_tape.kenga ===
bootstrap\bin\kenga-lite.exe run kenga\emit\rt_kval_tape.kenga
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
  for %%f in (lower_str_kv lower_events_kv lower_agent_kv lower_lex_frag lower_parse_frag lower_kv_mem lower_kv_tensor lower_kv_tape lower_kv_mlp lower_kv_ce lower_kv_living bc_vm_seed bc_while_sum bc_from_src bc_from_for bc_from_fn bc_from_lists bc_from_for_list bc_from_break bc_from_for_lite bc_from_lists_lite bc_from_elif bc_from_elif_lite bc_from_float_lite bc_from_struct_lite bc_from_agent bc_from_import bc_from_arith bc_from_fact_lite bc_from_str bc_from_net bc_from_tensor bc_from_mem bc_from_tape bc_from_tape_ops) do (
    cl /nologo /O2 /TC %%f.c /Fe:%%f.exe /Fo:%%f.obj
    if errorlevel 1 exit /b 1
    %%f.exe
    if errorlevel 1 exit /b 1
  )
  popd
  cl /nologo /O2 /TC bootstrap\generated\bc_from_io.c /Fe:bootstrap\generated\bc_from_io.exe /Fo:bootstrap\generated\bc_from_io.obj
  if errorlevel 1 exit /b 1
  bootstrap\generated\bc_from_io.exe
  if errorlevel 1 exit /b 1
  cl /nologo /O2 /TC bootstrap\generated\bc_from_birth.c /Fe:bootstrap\generated\bc_from_birth.exe /Fo:bootstrap\generated\bc_from_birth.obj
  if errorlevel 1 exit /b 1
  bootstrap\generated\bc_from_birth.exe
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
    for %%f in (lower_fact lower_for lower_if lower_fn lower_list lower_str lower_events lower_agent lower_import lower_for_lite lower_lists_lite lower_struct_lite lower_elif_lite lower_float_lite) do (
      gcc -O2 -std=c99 bootstrap\bin\%%f.c -o bootstrap\bin\%%f.exe
      if errorlevel 1 exit /b 1
      bootstrap\bin\%%f.exe
      if errorlevel 1 exit /b 1
    )
    pushd bootstrap\generated
    for %%f in (lower_str_kv lower_events_kv lower_agent_kv lower_lex_frag lower_parse_frag lower_kv_mem lower_kv_tensor lower_kv_tape lower_kv_mlp lower_kv_ce lower_kv_living bc_vm_seed bc_while_sum bc_from_src bc_from_for bc_from_fn bc_from_lists bc_from_for_list bc_from_break bc_from_for_lite bc_from_lists_lite bc_from_elif bc_from_elif_lite bc_from_float_lite bc_from_struct_lite bc_from_agent bc_from_import bc_from_arith bc_from_fact_lite bc_from_str bc_from_net bc_from_tensor bc_from_mem bc_from_tape bc_from_tape_ops) do (
      gcc -O2 -std=c99 %%f.c -o %%f.exe
      if errorlevel 1 exit /b 1
      %%f.exe
      if errorlevel 1 exit /b 1
    )
    popd
    gcc -O2 -std=c99 bootstrap\generated\bc_from_io.c -o bootstrap\generated\bc_from_io.exe
    if errorlevel 1 exit /b 1
    bootstrap\generated\bc_from_io.exe
    if errorlevel 1 exit /b 1
    gcc -O2 -std=c99 bootstrap\generated\bc_from_birth.c -o bootstrap\generated\bc_from_birth.exe
    if errorlevel 1 exit /b 1
    bootstrap\generated\bc_from_birth.exe
    if errorlevel 1 exit /b 1
  )
)

echo === examples/ml/kenga_trigram.kenga ===
bootstrap\bin\kenga-lite.exe run examples\ml\kenga_trigram.kenga
if errorlevel 1 exit /b 1

echo.
echo OK: freedom smoke ^(more + lower_c + KVal lower_kv → native^)
