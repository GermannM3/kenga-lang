# Kenga Freestanding

emit-c --freestanding → C99 без libc. Канон: docs/FREESTANDING.md

## Воспроизведение

`
bootstrap\build.cmd
`

Дальше emit с --freestanding. Проверка: RUNTIME_FS без stdio/stdlib, есть kf_alloc.

## Честно

Не ОС, не RTOS cert, не CUDA. Хуки ядра пишете вы.
