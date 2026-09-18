# WorkLog

單檔可執行的桌面工時紀錄小工具。資料存在內嵌 SQLite，不需登入、不需安裝資料庫。

## 功能

- 多使用者，不需登入；刪除使用者時紀錄保留並轉為無擁有者
- 紀錄欄位：標題、內容、工單、狀態、工作日期、工時、關聯工作
- 一個欄位同時模糊搜尋標題、內容、工單；可加日期區間、狀態複選、使用者過濾
- 日期快捷：今日、本週、本月、本 sprint、上個 sprint，與起訖日期雙向連動
- 每位使用者可設每日必填工時，扣除週末、國定假日與請假後算出缺口
- 缺口提示在搜尋列與系統匣同時顯示，不足、剛好、超出三種都會說
- 介面語言繁體中文與 English 可切；深、淺、藍三套主題

## 開發者手冊

環境怎麼裝、怎麼 debug、怎麼建置、怎麼使用，全部在 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)。

## 環境需求

- Visual Studio Build Tools，勾「使用 C++ 的桌面開發」（Rust 的連結器來自這裡，漏了一定編譯失敗）
- Rust：`winget install Rustlang.Rustup` 後 `rustup default stable-msvc`
- Tauri CLI：`cargo install tauri-cli --version "^2"`
- WebView2：Windows 11 內建，不需另裝

## 開發

```
cd src-tauri
cargo tauri dev
```

## 測試

```
cd src-tauri
cargo test
```

## 建置免安裝執行檔

```
cd src-tauri
cargo tauri build
```

產物在 `src-tauri/target/release/WorkLog.exe`，單一檔案、雙擊即開，不產生安裝程式。

## 資料庫

預設建在執行檔同目錄的 `worklog.db`，整個資料夾可以複製到隨身碟帶著走。執行檔放在 `Program Files` 這類沒有寫入權限的位置時，改用設定面的資料分頁切換到別的路徑。
