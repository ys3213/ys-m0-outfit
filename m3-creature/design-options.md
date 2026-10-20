# 介面與生物細節：第二輪候選

日期：2026-10-20。狀態：本人已選 C 柔體動態＋A 薄膜材質＋D 標本介面。

本人希望介面更精緻，生物更奇異、夢幻，且外形與生存方式能對應多姆山低層雲霧環境。先找 GitHub 原始專案，再由本人選擇；本次候選不是已定案物種，名稱亦非正式更名。

## 四個可混搭方向

| 候選 | 外觀／介面提案 | 可以移植的原始部分 | 與環境的對應假說 |
|---|---|---|---|
| A 珠光膜囊／暗色展場 | 通透雙層膜、不對稱囊室、內部顆粒；深灰雲霧與少量文字 | Glass Flower 的通透材質、薄膜彩邊與光源配置；以自己的生物幾何代替範例模型 | 凝膠保護層、濕潤囊室作保水候選；不設定為沉重玻璃或礦石 |
| B 異形褶花／編輯式展場 | 薄瓣變為錯落褶邊，乾濕時收展；格線與局部放大 | WebGLBlobs 的幾何形變、色帶／光影與版面結構 | 可變形表面和導水溝槽；接觸面增加也可能加快失水，不能只增加優點 |
| C 霧中游絲／沉浸雲海 | 褶邊外罩、分岔細絲、網狀內紋與不同延遲的柔體擺動；點選才顯示細節 | Aurelia 的參數形體、Verlet 柔體與程序紋理思路；以 Rust 重建適合本作品的部分 | 借用柔體視覺描繪微型濕潤包被與細絲；移動由氣流搬運，不能照搬水母水中推進 |
| D 微光標本／近距離觀察 | 淺霧灰、單一大主體、細字標記與懸停揭露；可配 A／B／C 的身體 | Frosted Glass 的單物件構圖、阻尼跟隨、霧面揭露與資訊動畫；不用鞋款模型及品牌 | 表面／內部／生命週期切換，讓膜脈、囊室與散播階段能被觀察；這是介面，不另指定生物祖先 |

建議比較組合：C 的柔體動態＋A 的薄膜材質＋D 的觀察介面。本人其後選擇此組合；不將協作建議延伸成未表達過的個人理由。

## GitHub 原始來源、授權與實際開啟

1. Poimandres **Glass Flower**：
   - [原始碼](https://github.com/pmndrs/examples/tree/main/examples/glass-flower)
   - [實際示範](https://pmndrs.github.io/examples/glass-flower/)
   - [examples MIT](https://github.com/pmndrs/examples/blob/main/LICENSE)、[drei MIT](https://github.com/pmndrs/drei/blob/master/LICENSE)
   - 已讀 App.tsx：MeshTransmissionMaterial、iridescence、燈光、Bloom、色調。React／Three.js 材質無法直接貼進目前 Rust，必須移植合適的 shader 與場景配置；不承諾套件即插即用。
2. Mario Carrillo／Codrops **WebGLBlobs**：
   - [原始碼](https://github.com/codrops/WebGLBlobs)
   - [實際示範](https://tympanus.net/Tutorials/WebGLBlobs/)
   - [MIT](https://github.com/codrops/WebGLBlobs/blob/main/LICENSE)
   - 已讀 README 與實際頁面。借用幾何形變與版面節奏；原作字型、文字及其他素材不自動沿用。
3. Holtsetio **Aurelia**：
   - [原作者原始碼](https://github.com/holtsetio/aurelia)
   - [實際示範](https://holtsetio.com/lab/aurelia/)
   - [MIT](https://github.com/holtsetio/aurelia/blob/master/LICENSE)
   - 已從搜尋所得 fork 追到原作者；讀 README 與程式檔案結構。原作是 Three.js WebGPU，展示正常；目前作品是 Rust WebGL2，需移植參數建模與柔體演算法，不直接搬整個 WebGPU 渲染器。
4. Poimandres **Frosted Glass**：
   - [原始碼](https://github.com/pmndrs/examples/tree/main/examples/frosted-glass)
   - [實際示範](https://pmndrs.github.io/examples/frosted-glass/)
   - 同 examples MIT；已讀 App.tsx 與 Overlay.tsx。鞋模型註明 quaz30／CC-BY-4.0；本作品只採介面與程式方法，另用自有生物，不沿用鞋、品牌與產品文案。

四個示範皆在 Chromium 1200 × 800 實際開啟，HTTP 200、有 canvas，未捕獲頁面例外；實際截圖留於本機選型頁。這是參考頁讀取檢查，不是對第三方所有功能的測試，也不能從外觀判定原作者是否用過 AI。

## 生存合理性：先建立限制，再增加形態

### 已查證事實

- **雲水微生物**：Amato et al. (2017) 以多姆山雲水 DNA／RNA 分析支持部分微生物具有活動跡象，並討論地表來源及濕沉降；不證明巨型空中動物存在。[PLOS 原始論文](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0182869)
- **保護與代謝的成本**：Amato et al. (2019) 轉錄體研究報告抗氧化、低溫／滲透壓回應與多醣相關活動；胞外多醣可作保護層研究參照，但不能把基因表現直接當成某種厚膜已被量測。[原始論文](https://www.nature.com/articles/s41598-019-41032-4)
- **地面到空氣的機制**：Joung et al. (2017) 實驗顯示雨滴撞擊土壤時，氣泡破裂可將含細菌微滴送進空氣；成效依土壤及條件而異。不證明 20–100 微米的完整複雜個體也能用同樣效率上升。[原始論文](https://www.nature.com/articles/ncomms14668)
- **導水紋理類比**：Ju et al. (2012) 研究仙人掌刺的圓錐形、溝槽與毛狀結構如何參與霧水收集。僅支持表面結構可影響水滴移動；不把陸生植物的組織或尺度直接移植成微生物器官。[原始論文](https://www.nature.com/articles/ncomms2253)
- **獨立研究的限制提醒**：Huss et al. (2026) 研究另一仙人掌物種，發現濕度驅動形變有助收集霧水，但沒有直接經刺進入活體皮層的吸收途徑；水沿表面流向根部。故「收集到水」與「把水吸收入體內」需分開設計。[原始論文](https://doi.org/10.1111/nph.71648)

### 合理推論／待本人討論

將「地面濕潤階段生長 → 較小的散播階段進入空氣 → 雲水中短暫活動 → 隨雨回落」作為生命週期候選，比永久漂浮且一直取得養分更能對上已知限制。地面濕潤位置、散播體尺寸、保水物質與能量收支仍需選定後細查。

### 藝術創作設定

雙層膜、囊室、褶邊、彩色光澤及分岔細絲是視覺提案；尚非已證實的適應。以放大觀察的微尺度生命延續既有作品，20–100 微米仍是先前暫設尺度，不能因畫面細緻就宣稱可以容納所有器官。光澤優先作反射／干涉視覺，不直接增加無限發光的能量來源。500 年是敘事時間，不是足以演化出新構造的證據。

## 查證過程與來源限制

- SIFT：辨認原作者／出版機構、讀原始研究而非新聞摘要；從 fork 回溯 Aurelia 上游；比較不同研究團隊對霧水收集和吸收的界線，不跨物種直接套用。
- Nature 出版社頁在搜尋工具遇 cookie 轉址；改以直接公開 HTTP 取得 HTML，成功讀到三篇原文的摘要與相關段落。PMC 副本遇驗證頁，未以該頁作查證終點。
- WebGLBlobs 最初嘗試 Development 路徑讀取失敗；依 README 改為正確 Tutorials 路徑後成功。
- DGFX/codrops-dreamy-particles 可讀 repo 及示範，但 GitHub license API 回 404，未能確認可直接重用的程式授權；不列正式採用候選。
- 選型階段未改動已發布程式。其後本人已選擇 C 柔體動態＋A 薄膜材質＋D 標本介面；實作對照見 UI-SOURCES.md 與 biology-design.md。

## 本人定案回覆

「C 的柔體動態＋A 的薄膜材質＋D 的標本介面（建議）」——本次依此組合實作，未另行替本人決定名稱或生命週期。
