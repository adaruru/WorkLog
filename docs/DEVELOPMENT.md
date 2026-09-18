# WorkLog 開發者手冊

從零把環境架起來、跑起來、改下去、包成 exe。照順序做就好。

## 1. 環境準備

只需要裝三樣東西，全部一次性。

### 1.1 Visual Studio Build Tools（最容易漏掉的一步）

Rust 在 Windows 上走 MSVC，連結器 `link.exe` 來自這裡。**沒裝的話 Rust 會裝得起來但編譯一定失敗**。

```
winget install Microsoft.VisualStudio.2022.BuildTools
```

裝完打開 Visual Studio Installer，勾選「**使用 C++ 的桌面開發**」（Desktop development with C++），套用。這一步會下載約 2–3 GB。

### 1.2 Rust

```
winget install Rustlang.Rustup
rustup default stable-msvc
```

新開一個終端機再驗證，`cargo` 要在 PATH 裡才算成功：

```
cargo --version
rustc --version
```

### 1.3 Tauri CLI

```
cargo install tauri-cli --version "^2"
```

這步會編譯一陣子（五到十分鐘正常）。裝完驗證：

```
cargo tauri --version
```

### 1.4 WebView2

Windows 11 內建，不用裝。介面就是跑在這個 WebView 裡。

## 2. 專案結構

```
WorkLog/
├─ src/                    前端：純 HTML + CSS + JS，無框架、無 npm
├─ src-tauri/              後端：Rust
│  ├─ src/
│  ├─ capabilities/
│  ├─ Cargo.toml
│  └─ tauri.conf.json
├─ docs/
└─ README.md
```

前端各檔的職責：

- `index.html` — 主畫面骨架：搜尋列、左列表、分隔線、右詳情、兩個 dialog。
- `settings.html` — 設定視窗骨架：六個分頁。
- `styles.css` — 設計系統。最上面是三套主題的 CSS 變數，改配色只動那三個區塊。
- `i18n.js` — 語言字典與 `data-i18n` 套用機制。加字串就加在 `DICT` 的兩份表裡。
- `api.js` — 所有 `invoke` 呼叫的包裝，外加 toast、主題套用、格式化小工具。
- `app.js` — 主畫面全部邏輯：查詢、列表、詳情、關聯、缺口徽章、分隔線拖曳。
- `settings.js` — 設定視窗全部邏輯。

Rust 各檔的職責：

- `main.rs` — 進入點：開資料庫、註冊 command、視窗尺寸記憶、關窗行為（見 3.4）。
- `db.rs` — schema DDL、內建狀態與預設設定的 seed、版本升級。
- `models.rs` — 所有跨層資料結構，`serde` 序列化給前端。
- `query.rs` — 查詢條件組裝成 SQL 片段，純函式，測試最好寫。
- `store.rs` — 所有資料存取與缺口計算的組裝。
- `calendar.rs` — 工作日判定、請假扣抵、缺口加總、sprint 推算。純邏輯，沒有資料庫。
- `commands.rs` — `#[tauri::command]` 薄層，只做參數檢查與錯誤轉字串。
- `tray.rs` — 系統匣圖示（RGBA 在程式裡畫出來，沒有 .ico 檔）與選單。

## 3. 開發循環

### 3.1 啟動

```shell
# cd D:\git\WorkLog\src-tauri
cd src-tauri
cargo tauri dev
```

第一次會編譯全部相依套件，五到十五分鐘，之後的增量編譯是幾秒鐘。

### 3.2 改前端

`src/` 底下的 HTML、CSS、JS 改完，**在 app 視窗按 `Ctrl+R` 重新載入**即可，不用重啟。

這個專案沒有 dev server、沒有 HMR（`tauri.conf.json` 的 `frontendDist` 直接指向 `../src` 靜態目錄），所以不會自動刷新，要自己按。

### 3.3 改 Rust

存檔後 `cargo tauri dev` 會自動重新編譯並重啟整個 app。改 Rust 一定會重啟，前端狀態會歸零。

### 3.4 debug 與正式版的一個行為差異

關掉主視窗時：

- **debug build**：程式直接結束。這樣下一次編譯才不會因為舊實例鎖著 `worklog.exe` 而失敗。
- **正式版**：縮到系統匣繼續常駐提醒，要真的離開得從系統匣圖示選「離開」。

所以**系統匣常駐這個行為要用 `cargo tauri build` 出來的 exe 驗收**，`cargo tauri dev` 下看不到。

### 3.5 加一個新功能的典型路徑

1. `models.rs` 加資料結構。
2. `store.rs` 加存取函式，順手在同檔 `mod tests` 加測試。
3. `commands.rs` 包成 command。
4. `main.rs` 的 `invoke_handler!` 清單裡加上它（**漏了這行，前端會收到 command not found**）。
5. `api.js` 加對應包裝。
6. `app.js` 或 `settings.js` 接上介面。
7. `i18n.js` 兩份字典都補上新文案。

## 4. Debug

### 4.1 前端

在 app 視窗按右鍵 → 檢查（Inspect），或 `F12`，開出來就是 Chromium DevTools。Console、Network、Elements 都能用，斷點也能下。

debug build 預設就啟用 DevTools，release build 不會。

前端印訊息就用 `console.log`，看得到 `invoke` 的回傳長什麼樣。

### 4.2 Rust

`println!` 或 `dbg!` 的輸出會直接出現在跑 `cargo tauri dev` 的那個終端機。

command 回傳 `Err(String)` 時，前端 `UI.guard` 會把訊息顯示成紅色 toast，不用自己處理。

### 4.3 資料庫

開發時資料庫不在專案根目錄，而是在**執行檔旁邊**：

```
src-tauri/target/debug/worklog.db
```

要看內容用 [DB Browser for SQLite](https://sqlitebrowser.org/) 打開它。

想從乾淨狀態重來，把那個檔刪掉再跑一次，啟動時會自動重建 schema 與四筆內建狀態。

### 4.4 常見卡點

- **`link.exe not found` 或 `error: linker not found`** — Visual Studio Build Tools 沒裝，或裝了但沒勾「使用 C++ 的桌面開發」。回到 1.1。
- **`rustup` 或 `cargo` 不是可辨識的命令** — 裝完沒開新終端機，PATH 還是舊的。關掉重開就好。不想重開的話，在當前 session 補一次：`$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"`，或直接用完整路徑 `& "$env:USERPROFILE\.cargo\bin\rustup.exe"`。
- **`` `icons/icon.ico` not found; required for generating a Windows Resource file ``** — Windows 上 tauri-build 一定要這個檔，就算 `bundle.active` 是 `false` 也一樣。用官方工具從一張 1024×1024 的 PNG 產生：`cargo tauri icon path\to\logo.png`。它會連 Android、iOS、Microsoft Store 的圖示一起產出來，這個專案只需要 `icon.ico`、`icon.png`、`32x32.png`、`128x128.png` 四個，其餘可以刪掉。
- **前端改了畫面沒變** — 沒按 `Ctrl+R`。這個專案不會自動刷新。
- **DevTools 看到 CSP 錯誤，圖片或字型載不進來** — `tauri.conf.json` 的 `security.csp` 限制了來源。這個 app 刻意不吃任何外部資源，要加東西請改用 inline SVG 或 data URI。
- **`command not found` 之類的 invoke 錯誤** — `main.rs` 的 `invoke_handler!` 清單漏加。
- **改了 command 參數名，前端傳過去卻收到 null** — Tauri 會把前端的 camelCase 參數名轉成 Rust 的 snake_case，但**只對 command 的參數生效**。傳進去的物件（像 `EntryQuery`、`EntryInput`）內部欄位要自己寫 snake_case。
- **`` error: failed to remove file `target\debug\worklog.exe` ``** — 上一個 app 實例還活著，鎖住了執行檔。結束它：`Stop-Process -Name worklog -Force`。正式版關掉視窗只會縮到系統匣、程式不會結束，就是這個原因；debug build 已經改成關掉視窗即結束，所以開發時通常不會遇到。
- **系統的全域熱鍵在 app 有焦點時失效（例如截圖工具的 F1）** — WebView2 預設把 F1、F3、F5、F7、Ctrl+F、Ctrl+P 當成瀏覽器快捷鍵吃掉。`main.rs` 的 `release_browser_hotkeys` 已在視窗建立後關閉 `AreBrowserAcceleratorKeysEnabled`；若之後改動了視窗建立流程導致這段沒被呼叫，症狀就會回來。
- **資料庫顯示 locked** — 同時開了兩個 app 實例，或 DB Browser 正壓著沒存檔的變更。關掉其中一邊。

## 5. 測試

```
cd src-tauri
cargo test
```

只跑某一群：

```
cargo test calendar        跑 calendar 模組
cargo test sprint          跑名字含 sprint 的測試
cargo test -- --nocapture  讓 println! 的輸出顯示出來
```

測試都跟著程式放在同一個檔的 `#[cfg(test)] mod tests`。資料層測試用 `db::open_in_memory()` 開記憶體資料庫，不會碰到任何實體檔案，可以放心跑。

寫新測試時，純邏輯（日期、缺口、SQL 組裝）優先寫在 `calendar.rs` 與 `query.rs`，那兩個檔沒有 I/O，測起來最快也最穩。

## 6. 建置正式版

```
cd src-tauri
cargo tauri build
```

產物：

```
src-tauri/target/release/WorkLog.exe
```

這支 exe 就是完整的程式，雙擊即開，不需要安裝、不需要額外的 DLL、不會產生安裝程式（`tauri.conf.json` 裡 `bundle.active` 是 `false`）。

發佈時把 exe 複製到任何資料夾即可，第一次執行會在它旁邊建 `worklog.db`。

## 7. 怎麼使用

### 7.1 第一次啟動

畫面會是空的，照這個順序設定：

1. 右上角齒輪打開設定。
2. **使用者**分頁：新增自己，填每日必填工時（預設 8）。
3. **Sprint** 分頁：填一個你知道起始日的 sprint，例如編號 `12`、起始日 `2026-09-01`、每 sprint `14` 天。填完之後所有 sprint 的起訖日都能自動算出來。
4. **假日與請假**分頁：把國定假日加進去，這些日子不會算進應填工時。請假也在這裡登記，可以只請半天（填 4）。
5. **一般**分頁：挑語言、主題，還有提醒範圍要看今日、本週還是本 sprint。
6. 關掉設定，回主畫面左上角的使用者下拉選自己，缺口徽章就會開始運作。

### 7.2 日常操作

- **記一筆**：按「新增」，填標題與工時，`Ctrl+S` 存檔。
- **改狀態**：不用進詳情，直接在左方列表那一列的下拉改，改完立刻存。
- **找東西**：搜尋框一個欄位同時比對標題、內容、工單。
- **看某段期間**：日期快捷選今日／本週／本月／本 sprint／上個 sprint，起訖日期會自動填；反過來手動改日期，快捷會切成「自訂」。
- **關聯工作**：在詳情下方按「加關聯」，清單依最近更新排序。關聯建立後，被關聯的那一筆在自己的「關聯我的」區塊也看得到，點任一邊都會導航過去。
- **調整版面**：拖動中間那條分隔線改左列表寬度，關掉視窗會記住。
- **背景常駐**：關視窗不會結束程式，縮到系統匣繼續提醒。要真的離開，在系統匣圖示按右鍵選離開。

### 7.3 快捷鍵

- `Ctrl+S` — 儲存目前編輯的紀錄
- `Ctrl+N` — 新增一筆
- `Ctrl+R` — 重新載入畫面（開發時用）
- `F12` — 開 DevTools（debug build）

## 8. 資料放哪

預設是執行檔同目錄的 `worklog.db`，整個資料夾複製到隨身碟就能帶著走。

如果把 exe 放在 `Program Files` 這類沒有寫入權限的位置，啟動會失敗或存不進東西，這時到設定的**資料**分頁把路徑改到有權限的地方。
