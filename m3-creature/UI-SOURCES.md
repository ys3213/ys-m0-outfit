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

## 第二輪：C 動態＋A 薄膜＋D 標本介面（本人已選）

本輪的 A／C／D 編號與第一輪不同，完整候選見 [design-options.md](design-options.md)。第一輪紀錄與 Codrops 原始 CSS 保留。

| 採用部分 | 原始碼與版本 | 實際改作範圍 |
|---|---|---|
| C：Aurelia 柔體外罩 | [Holtsetio / aurelia](https://github.com/holtsetio/aurelia)，commit `724aba263459d1b11a49b72bfc7ce8baac840ca8` | `getBellPosition` 的相位延遲、極角、徑向波動與邊緣褶皺公式改寫為 `soft_body.rs::bell`。修改振幅、開合、五瓣輪廓、座標與含水控制；保留原始 JS 和 MIT 在 vendor/aurelia。沒有載入原作 WebGPU／Three.js。 |
| C：細絲動態 | 同上 `medusaTentacles.js`、`physics/verletPhysics.js` | 參考原作固定根部、串接節點與細管方式；本作另寫 CPU 位置式 Verlet 與節長限制，增加分岔。不是宣稱原作整套 GPU 彈簧系統已移植，也不是 IK。 |
| A：通透珠光材質 | [pmndrs Glass Flower](https://github.com/pmndrs/examples/tree/9905cd7b3d030a0ba5c6d327e15fe30ab648abb6/examples/glass-flower) | 依其雙面透光、彩邊與柔光方向寫 WebGL2 薄膜近似；依視角著色、平滑法線、鏡面亮點、半透明三角形排序。**沒有直接移植 MeshTransmissionMaterial**，沒有承諾真實折射、色散或相同渲染結果。 |
| D：資訊展開與構圖 | [pmndrs Frosted Glass Overlay.tsx](https://github.com/pmndrs/examples/blob/9905cd7b3d030a0ba5c6d327e15fe30ab648abb6/examples/frosted-glass/src/Overlay.tsx) | 將原作 hidden/show 與逐項 0.05 秒錯開的資訊揭露改為 Rust 開關＋CSS 動畫。採單一大標本、邊欄小標記與霧面面板；原始 Overlay 與 MIT 存在 vendor/pmndrs。新增全貌／外膜／囊室／細絲切換，不沿用鞋子、品牌、售價、Logo 或 GLB。 |

- 生物幾何、分岔、囊室與液珠由程式即時計算，沒有使用網路模型、外部貼圖或生成圖片。
- 上游程式只留作來源對照，不由網頁載入；互動和繪圖依然在 Rust／WebAssembly。
- 舊 p5.js 仍在 m2-species；第一版 Rust 也可在 Git 紀錄 [53dfd4e](https://github.com/ys3213/ys-m0-outfit/tree/53dfd4e6e7ca7b52f169e0bbca0ce68c49329e99/m3-creature) 查看。官方課程文字與既有研究、簡報未替換。
- 外觀不作為「有無使用 AI」的證明；實際 AI 協作繼續記在本里程碑 README。
