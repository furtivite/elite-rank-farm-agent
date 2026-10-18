# Changelog · Список изменений

What changed in the agent from version to version — for those who download
it. Every release ships with its section from here: without it the release
is not created.

Что менялось в агенте от версии к версии — для тех, кто его скачивает.
Каждый релиз выходит со своим разделом отсюда: без него релиз не
создаётся.

Section format · формат раздела: `## <version> — <date>`, then `### English`
and `### Русский`.

## 0.6.5 — 2026-10-18

### English

- The library the agent uses for its encrypted connection to the site
  (rustls) is updated to fix a flaw in how such connections are set up.
  Nothing changes in how the agent works.
- The release page now lists the SHA-256 of `elite-agent.exe`, so you can
  check the file you downloaded.

### Русский

- Обновлена библиотека, через которую агент держит зашифрованное соединение
  с сайтом (rustls): в ней исправлена ошибка установки такого соединения.
  В работе агента ничего не меняется.
- На странице релиза теперь есть SHA-256 файла `elite-agent.exe` — им можно
  проверить скачанное.

## 0.6.4 — 2026-10-18

### English

- The window can no longer be made narrower than its content: it stops at
  the width of the widest button, so nothing is cut off at the right edge.
  The limit follows the chosen language.
- The suit in the window is named in the window’s language: “Maverick Suit”
  in English even when the game itself runs in another language. A suit the
  game left untranslated now gets its real name (“Artemis Suit”) instead of
  an internal code.

### Русский

- Окно больше не сужается уже своего содержимого: оно останавливается на
  ширине самой широкой кнопки, и справа ничего не обрезается. Предел
  зависит от выбранного языка.
- Скафандр в окне называется на языке окна: в английском окне — «Maverick
  Suit», даже если сама игра на русском. Скафандр, который игра не
  перевела, получает настоящее название («Комбинезон Artemis») вместо
  внутреннего кода.

## 0.6.3 — 2026-10-18

### English

- **The agent speaks English by default.** Russian can be chosen in the
  settings (⚙ → “Language · Язык”); the choice is kept between starts, in
  `%APPDATA%\elite-agent` with the rest of the agent files.
- Switching takes effect at once — the window and the output — without a
  restart.
- When a newer version is out, the agent now also says where to download it.
- **Collect and Mining missions know what you carry.** The agent checks the
  hold against such missions and tells the site how much of the commodity is
  aboard — only that number, not your cargo list. Until the goods are
  aboard, the basket keeps the mission but does not send you to hand it in.
- Versions 0.6.0–0.6.2 keep working with the site: you can update when it
  suits you.

### Русский

- **Агент по умолчанию говорит по-английски.** Русский выбирается в
  настройках (⚙ → «Language · Язык»); выбор сохраняется между запусками — в
  `%APPDATA%\elite-agent`, вместе с остальными файлами агента.
- Язык меняется сразу — и в окне, и в выводе — без перезапуска.
- Когда вышла новая версия, агент теперь говорит и где её скачать.
- **Задания Collect и Mining знают, что у вас в трюме.** Агент сверяет трюм
  с такими заданиями и сообщает сайту, сколько их товара на борту, — только
  это число, а не список груза. Пока товар не набран, корзина держит задание,
  но сдавать его не зовёт.
- Версии 0.6.0–0.6.2 продолжают работать с сайтом: обновиться можно, когда
  удобно.

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
