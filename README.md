# Horizon Ore Chests

A native Rust mod for [Horizon Launcher](https://github.com/horizon-rust/horizon), porting the classic **Iron Chests** mod to Minecraft 26.3 with tiered, expandable, upgradable, and lockable chests.

---

## 📦 Features / Возможности

* **10 Уровней Сундуков (10 Chest Tiers):**
  * 🪵 **Wood Chest** (Vanilla): 27 слотов (9×3)
  * 🟫 **Dirt Chest 9000**: 126 слотов (14×9) — принимает только землю (`minecraft:dirt`)
  * 🟧 **Copper Chest**: 45 слотов (9×5)
  * ⬜ **Iron Chest**: 54 слота (9×6)
  * 🟨 **Gold Chest**: 81 слот (9×9)
  * 💎 **Diamond Chest**: 108 слотов (12×9)
  * 🟩 **Emerald Chest**: 108 слотов (12×9)
  * 🔮 **Crystal Chest**: 108 слотов (12×9) — прозрачный корпус, отображает хранящиеся внутри предметы
  * ⬛ **Obsidian Chest**: 108 слотов (12×9) — взрывоустойчивый (прочность обсидиана)
  * 🔥 **Netherite Chest**: 126 слотов (14×9) — огнеупорный, не горит в лаве, взрывоустойчивый
  * 🎁 **Christmas Chest**: 27 слотов (9×3) — праздничное оформление

* **⚡ Улучшения на месте (In-Place Upgrades):**
  Улучшайте уже установленные сундуки без необходимости их ломать и выгружать предметы! Нажмите ПКМ (или Shift+ПКМ) соответствующим предметом улучшения:
  * **Copper Upgrade**: Wood → Copper
  * **Iron Upgrade**: Copper → Iron
  * **Gold Upgrade**: Iron → Gold
  * **Diamond Upgrade**: Gold → Diamond
  * **Emerald Upgrade**: Gold → Emerald
  * **Crystal Upgrade**: Diamond / Emerald → Crystal
  * **Obsidian Upgrade**: Diamond / Emerald → Obsidian
  * **Netherite Upgrade**: Diamond / Emerald / Obsidian → Netherite

* **🔒 Ключ и Блокировка (Container Key):**
  * Запирайте сундуки персональным ключом (`orechests:container_key`).
  * Защита от воровства и несанкционированного доступа.
  * Настройка доверенных игроков (trusted list).

* **🛒 Тележка для переноски (Hand Truck):**
  * **Hand Truck**: позволяет поднять сундук вместе со всем его содержимым и перенести на новое место.
  * **Enhanced Hand Truck**: продвинутая версия, позволяющая также перемещать спавнеры мобов.

---

## 🚀 Установка (Installation)

1. Откройте **Horizon Launcher** → перейдите во вкладку **Mods** → **Horizon Mods**.
2. В поле **"Add native mod from GitHub"** вставьте ссылку:
   ```text
   https://github.com/swwgg/horizon-OreChests
   ```
3. Нажмите **Add from GitHub**, затем нажмите **Build**.

---

## ⚙️ Конфигурация (Configuration)

Настройки мира генерируются автоматически при первом запуске:
`<world>/horizon-orechests/config.json`

Параметры:
* `open_under_solid_blocks` (по умолчанию: `false`): открывать ли сундуки под сплошными блоками.
* `copper_golem_interaction` (по умолчанию: `true`): взаимодействие медных големов с сундуками.
* `locks_enabled` (по умолчанию: `true`): включена ли система блокировки ключом.
* `moving_spawners_enabled` (по умолчанию: `true`): разрешено ли улучшенной тележке переносить спавнеры.
* `rows`: кастомное количество рядов для каждого типа сундука (от 1 до 12).

---

## 🛠 Сборка и тестирование (Development)

Сборка и прогон автономных тестов правил без необходимости запуска игры:
```sh
cargo test
```

Упаковка готовых пакетов `.hnmod` и `.hmod`:
```sh
python3 tools/package.py
```
Файлы будут собраны в папку `target/`:
* `target/orechests-1.0.0.hnmod` (нативный модуль Horizon)
* `target/orechests-1.0.0.hmod` (гибридный пакет с текстурами, моделями и рецептами)

---

## 📜 Credits & License

* Основано на оригинальном моде [Iron Chests](https://github.com/4nner/IronChests) от GatherToCraft (GPL-3.0 / MIT).
* Переписано на Rust для [Horizon Launcher](https://github.com/horizon-rust/horizon).
* Лицензия: MIT.
