# Changelog · Список изменений

What changed in the agent from version to version — for those who download
it. Every release ships with its section from here: without it the release
is not created.

Что менялось в агенте от версии к версии — для тех, кто его скачивает.
Каждый релиз выходит со своим разделом отсюда: без него релиз не
создаётся.

Section format · формат раздела: `## <version> — <date>`, then `### English`
and `### Русский`.

## 0.6.2 — 2026-10-17

### English

- The agent no longer puts its log and working files next to the program
  (on the desktop, if that is where the `.exe` is). Everything lives in one
  folder, `%APPDATA%\elite-agent`, together with the settings and keys. The
  “Logs folder” button («Папка с логами») opens it.
- Files of earlier versions move there by themselves on the first start: the
  agent remembers what it has already sent and does not send the history
  again.

### Русский

- Лог и рабочие файлы агента больше не появляются рядом с программой (то
  есть на рабочем столе, если `.exe` лежит там). Всё хранится в одной папке
  `%APPDATA%\elite-agent` — там же, где настройки и ключи. Кнопка «Папка с
  логами» открывает её.
- Файлы прежних версий переезжают туда сами при первом запуске: агент
  помнит, что уже отправил, и не отправляет историю заново.

## 0.6.1 — 2026-10-17

### English

- The large light rectangles around the copyright line at the bottom of the
  window are gone; a thin line sits above it instead.
- In a narrow window that line is no longer cut off — it wraps in full.

### Русский

- Убраны большие светлые прямоугольники вокруг строки о правообладателях
  внизу окна; над ней теперь узкая полоса.
- В узком окне эта строка больше не обрезается — переносится целиком.

## 0.6.0 — 2026-10-17

### English

- **A new game resets the ranks.** If you start the game over, the panel
  shows the ranks of the new game, not the previous highest ones.
- **The window is a column.** Buttons stand one under another at full width,
  and the window is narrow by default — as wide as the widest button. Text
  wraps when the window is narrowed.
- **Commander details in the window:** CMDR, SHIP, SUIT, SYSTEM, STATION.
- **“Send everything again”** («Отправить всё заново») in the settings (⚙)
  sends everything from the game journals to the server once more.
- On closing, the agent sends what it has not sent yet and asks the site to
  refresh the galaxy data.

### Русский

- **Новая игра сбрасывает звания.** Если начать игру заново, панель
  показывает звания новой игры, а не прежние максимальные.
- **Окно столбиком.** Кнопки стоят одна под другой во всю ширину, окно по
  умолчанию узкое — по самой широкой кнопке. Текст переносится, когда окно
  сужают.
- **Сведения о командире в окне:** CMDR, SHIP, SUIT, SYSTEM, STATION.
- **«Отправить всё заново»** в настройках (⚙): заново отправляет на сервер
  всё из журналов игры.
- При закрытии агент досылает то, что не успел отправить, и просит сайт
  обновить данные о галактике.
