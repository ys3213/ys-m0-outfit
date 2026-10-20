# 執行與展示

## 開啟

- 線上：<https://ys3213.github.io/ys-m0-outfit/m3-creature/>
- 本機：在 repository 啟動 `python3 -m http.server 8000`，瀏覽器開 `http://127.0.0.1:8000/m3-creature/`。
- 本版本是 Rust／WebAssembly，必須透過網頁服務載入；不要直接雙擊 HTML 當成正式測試。線上試玩不需在電腦安裝 Rust。
- M2 的 p5.js 版維持在 `m2-species/`；新舊版皆保留。

## 互動

1. 「個體」先觀察一隻。把游標停在膜囊旁，水霧會補水，晶瓣逐漸展開；接近水霧中心時放慢。
2. 點擊放出三維波紋。「轉動」模式拖曳看側面、背面；滾輪或遠近滑桿縮放。「正面」恢復視角。
3. 「降雨」補水八秒；「旋風」讓個體繞行。Shift 在水霧模式暫時帶起旋風。
4. 移開游標，水分逐漸減少，晶瓣收攏，活動變慢。
5. 切到「花園」保留 16 隻起始與資源充足時的出芽，最多 28 隻。小個體會逐漸長大；本版不模擬死亡與遺傳演化。
6. 空白鍵暫停／繼續，R 重新萌生；暫停後仍可轉動觀察。頁面下方有世界與生命說明。

觸控可點擊、用旋轉模式拖曳、使用滑桿；未宣稱已測過每一款手機。

## 編譯與讀程式

- `rust/src/life.rs`：Creature、Garden、含水／能量、轉向、沉降、聚集與出芽；原生測試不需要瀏覽器。
- `rust/src/browser.rs`：用 Rust 處理 WebGL2、晶瓣／膜囊／感絲、按鈕、滑鼠、鍵盤與動畫迴圈。
- `index.html`：網頁結構；內嵌 JavaScript 只載入 Wasm 並處理失敗提示。
- `exhibition.css`：本作品的排版覆寫；上游 CSS 與 MIT 保存在 vendor/codrops。
- `pkg/`：實際編譯產物（Wasm 與瀏覽器連接碼），可部署到 GitHub Pages。編譯器暫存 rust/target 已忽略。

需要 Rust、wasm32-unknown-unknown 目標，以及與 Cargo.lock 中 wasm-bindgen **相同版本**的 CLI。配置完成後，執行 `./build.sh`。首次建立 Cargo.lock 用 `cargo build --manifest-path rust/Cargo.toml --target wasm32-unknown-unknown --release`；之後以 lock 固定。

目前這台電腦的 Rust 工具獨立放在使用者目錄 `.local/share/ys-zoo-rust`，未更改 shell 設定。build.sh 在一般 cargo 不存在時會使用這個位置。

## 檢查與排錯

- 輸入：滑鼠補水、點擊、切換視角、降雨、旋風、暫停、重置；輸出：開合、位置、波紋、含水讀數與狀態。
- 空白畫面先看啟動提示；確認 WebGL2／圖形加速開啟，以及 pkg 中兩個檔案沒有漏傳。
- 網頁服務須能提供 `.wasm`；請以線上網址或本機 HTTP 開啟，避開 file:// 的載入限制。
- 若轉動時不能補水，先切回「水霧」。若停住，先看是否暫停。
- 執行版本若與編譯結果不符，重新整理，並確認網路請求抓到目前的 Wasm。

## 課程進度

這是 M3 的工作版，不代表已完成 11/12 發表。正式交件語言是否可用 Rust 替代課程 p5.js，須由老師確認；原 p5.js 可執行成果仍保留。本人反思、閱讀與課堂回饋尚未代填。
