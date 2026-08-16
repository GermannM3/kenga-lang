# Книга «Кенга»

Первое издание, 2026. Автор — Герман Янтарас. Тот же формат, что книга по Z-системе: A5, обложка, оглавление, факты и опровержения.

- PDF: `kenga_kniga_yantaras_v1.pdf`
- EPUB: `kenga_kniga_yantaras_v1.epub`
- Текст и вёрстка: `make_book.py`
- Обложка: `make_cover.py` → `cover.png`, `back.png`

```bat
python book\make_cover.py
python book\make_book.py
```

или `scripts\make-book.cmd`.

Нужны `reportlab`, `ebooklib`, `matplotlib`, `numpy`. Если факт в книге разошёлся с репозиторием — правь `make_book.py`, не PDF руками.
