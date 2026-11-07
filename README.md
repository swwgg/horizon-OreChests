# Horizon Ore Chests

A native Rust mod for [Horizon Launcher](https://github.com/horizon-rust/horizon) adding 5 tiered ore chests for Minecraft 26.3.

---

## 📦 Сундуки и крафт (Chests & Crafting)

Каждый сундук крафтится на верстаке: **обычный сундук (`minecraft:chest`, `minecraft:trapped_chest` или `minecraft:barrel`) в центре + 8 соответствующих слитков/алмазов вокруг**.

| Сундук | Слоты (сетка) | Материал вокруг сундука (8 шт.) | Особенности |
| :--- | :--- | :--- | :--- |
| **Copper Chest** (Медный) | 45 (9×5) | `minecraft:copper_ingot` | Прочность 3.0, взрывоустойчивость 6.0 |
| **Iron Chest** (Железный) | 54 (9×6) | `minecraft:iron_ingot` | Прочность 5.0, взрывоустойчивость 6.0 |
| **Gold Chest** (Золотой) | 81 (9×9) | `minecraft:gold_ingot` | Прочность 3.0, взрывоустойчивость 6.0 |
| **Diamond Chest** (Алмазный) | 108 (12×9) | `minecraft:diamond` | Прочность 5.0, взрывоустойчивость 6.0 |
| **Netherite Chest** (Незеритовый) | 126 (14×9) | 1 `minecraft:netherite_ingot` (сверху) + 7 `minecraft:obsidian` | Огнеупорный, взрывоустойчивость 1200.0 |

---

## 🚀 Установка (Installation)

1. Откройте **Horizon Launcher** → перейдите во вкладку **Mods** → **Horizon Mods**.
2. В поле **"Add native mod from GitHub"** вставьте ссылку:
   ```text
   https://github.com/swwgg/horizon-OreChests
   ```
3. Нажмите **Add from GitHub**, затем нажмите **Build**.

---

## 🛠 Сборка и тестирование (Development)

Прогон автономных тестов:
```sh
cargo test
```

Упаковка пакетов `.hnmod` и `.hmod`:
```sh
python3 tools/package.py
```
Файлы собираются в папку `target/`:
* `target/orechests-1.0.0.hnmod` (нативный модуль Horizon)
* `target/orechests-1.0.0.hmod` (контент-пакет с текстурами, моделями и рецептами)

---

## 📜 Credits & License

* Вдохновлено модом [Iron Chests](https://github.com/4nner/IronChests).
* Написано на Rust для [Horizon Launcher](https://github.com/horizon-rust/horizon).
* Лицензия: MIT.
