# 介面來源與選擇

日期：2026-10-20。本人選擇 A「藝術展場」。以下均為實際開啟的 GitHub 專案與原作者示範，不宣稱作者未使用 AI，也不將外觀作為作者身分或工具使用的證明。

| 候選 | GitHub | 實際讀取的示範 | 採用狀態 |
|---|---|---|---|
| A 藝術展場 | https://github.com/codrops/OnScrollTypographyAnimations | https://tympanus.net/Development/OnScrollTypographyAnimations/ | 本人選定；MIT |
| B 標本圖鑑 | https://github.com/codrops/3DGridContentPreview | https://tympanus.net/Development/3DGridContentPreview/ | 保留候選；MIT |
| C 簡潔觀測台 | https://github.com/rust-ui/leptos-ui | https://rust-ui.com/ | 未採用；repo 為 Leptos，現行網站顯示 Dioxus，不視為同一實作版本；MIT |

## 實際採用

- 上游 CSS：https://raw.githubusercontent.com/codrops/OnScrollTypographyAnimations/main/src/css/base.css
- 上游 MIT：https://raw.githubusercontent.com/codrops/OnScrollTypographyAnimations/main/LICENSE
- vendor/codrops/base.css 原樣保留，與本次下載位元組一致；授權保存在同資料夾 LICENSE。上游第 38 行原有行尾空白亦保留，git diff --check 對此一行的提醒不涉及本作品程式錯誤。
- 使用原範例的 frame、content、content__title、unbutton 等排版結構與按鈕基礎；exhibition.css 改為本作品的色彩、中文字體、留白與操作列。
- 原 index.js 的 effect 1 使用字體縮放／透明度及捲動揭露。本版在 Rust 裡計算段落標題的縮放，使用 CSS transform 呈現；沒有載入 GSAP、Lenis 或 Splitting。
- 不使用上游的 Adobe 字型、照片、文字內容或其他第三方圖片。字體使用瀏覽器／系統既有字體；生物由 Rust 即時繪圖。
- 此處是改作紀錄，不將原範例完整套用稱作本人的原創排版。

上游 base.css SHA-256：2423260e77101b09b7130499dfea5dbc6cd4c591865a3b9d9e87f447630d9a45

## 排除或讀取限制

- Atelier North：README 指向的 LICENSE 實際為 404，無法確認完整授權內容；未採用。
- ARWES：原作者示範頁查詢與直接讀取皆逾時，repo 亦聲明停止維護；未採用。
- Interlude：已讀 MIT 與示範，但不是本人選擇，未引入套件。
