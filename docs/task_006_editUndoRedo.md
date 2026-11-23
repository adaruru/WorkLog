# Task 006 Edit Undo Redo

## Status

Activate

## Rules

- 除標題（H1/H2…）用英文外，內文一律中文繁體書寫。
- first thing first: 所有工作都要在 hit limit 之前 更新 goal 確認完成進度，如果你知道當前 session token 快用完，就先更新 goal
- Wait Decision
  - 你不應該在對話中處理待確認事項，需要確認的東西應該要寫在 Wait Decision
  - 已經確認的內容不應該更新 Wait Decision
  - 已經確認的內容，跟規格相關，應該更新在 Spec
  - 已經確認的內容，跟工作相關，應該更新在 Goal
  - 已經確認的內容，是重要背景知識（使用者不懂、經過調查後得到），應更新在 Survey 現況調查
  - 已經確認的內容，應該融入文件
  - Wait Decision 只放待確認事項，不會有已經確認的事項更新在此區塊
- Goal（使用者掌管，agent 唯讀）
  - goal 是我要達到的目的，**goal 全部完成這單才可以關**；一個 goal 通常要很多 task 完成與驗證來支撐
  - agent **不可以**自行新增／刪除／改寫／勾選 goal，要動一律先取得我明確授意；達成與否用回報的，不要自己打勾
- Task（agent 掌管）
  - task 是一步一步、agent 照著做得動的工作指標，agent 可隨時新增、精簡、重排、勾選，不必請示
  - task 表就是唯一的即時進度追蹤器：每一項開始做／做完就直接去動那個 checkbox，一項一項按順序走
  - task 不再另開一份內部 TaskList 自己追、然後事後回頭對帳核對
  - task 只勾實際做完的，沒做的維持 [ ]
  - task 只留勾選清單、說明全部移到 Survey、保持簡潔
- Survey
  - 使用者表示不懂、透過對話回答、上網搜尋的資訊，更新在 Survey
  - 使用者不確定、經調查詢程式碼、規格書查得到的背景知識、資訊
  - 使用者表示不懂、且需深入調查/教學、或需大量方案比較深入思考的議題，另開 survey 檔（例：`Task/survey_AddMemberAccessLog.md`）

## CurrentJob

ai，你做你的工作，別刪我的 CurrentJob，這是使用者的 local prompt 草稿與 task/plan/skill 管理工作區，你(AI、Agent、Plan)只需要讀取不可修改 CurrentJob。

## Goal: Agent 執行歷程確認表

### goal 編輯行為的使用者歷程

- [ ] 編輯行為記進使用者歷程，`Ctrl+Z` 復原、`Ctrl+Y` 還原復原
- [ ] 涵蓋上方查詢資料與下拉選單的變更
- [ ] 涵蓋清單的逐列編輯與下方的詳細編輯
- [ ] 涵蓋設定的編輯

### task 執行（歷程基礎設施）

- [x] 建立歷程 stack 的資料結構與 push／undo／redo 介面
- [x] 綁定 `Ctrl+Z`／`Ctrl+Y`，與既有 `Ctrl+N` 的 keydown 處理共存
- [x] 輸入框內的原生 undo 與全域歷程的衝突處理
- [x] 歷程為空或已到底時的回饋

### task 執行（查詢條件）

- [x] 關鍵字、範圍下拉、起訖日、狀態過濾、使用者過濾的變更記進歷程
- [x] 排序欄位與方向的變更記進歷程
- [x] undo／redo 後同步 UI 控制項與 `last_filter`

### task 執行（資料編輯）

- [x] 逐列編輯的六個欄位（工單、標題、工時、狀態、日期、使用者）記進歷程
- [x] 清單列上的狀態下拉直接切換記進歷程
- [x] 詳情的內容 textarea 記進歷程
- [x] 關聯工單的加入與移出記進歷程
- [x] 刪除紀錄時清空歷程
- [x] 新增紀錄的歷程處理

### task 執行（設定編輯）

- [x] 一般設定（語言、主題、間距、提醒範圍、預設狀態）記進歷程
- [x] 使用者的新增、改名、改工時記進歷程；刪除改為清空歷程
- [x] 狀態的新增、改名、改色記進歷程；刪除改為清空歷程
- [x] 假日與請假的新增、一鍵匯入記進歷程；刪除改為清空歷程
- [x] sprint 基準的三個設定記成同一筆歷程項
- [x] 設定視窗與主視窗的歷程串接（push／undo／redo／clear／changed 五個 event）
- [x] 新增請假後回查 id，因為 `save_leave` 不回傳 id

### goal 空結果也能新增工單

- [ ] 查詢結果為空、顯示「這個條件沒有紀錄」時，也要有加號按鈕可以新增工單
- [ ] 按下加號新增的工單要看得到

### task 執行（空結果新增工單）

- [x] `styles.css`：加 `.empty.list-empty`，讓清單空狀態不再吃掉整個可視高度
- [x] `app.js`：空清單的 `.empty` 掛上 `list-empty`
- [x] `app.js`：`createEntry` 新增後若新工單不在結果中，放寬過濾條件再重查並提示
- [x] `app.js`：把 `navigateTo` 清過濾條件的四行抽成 `relaxFilters()` 兩處共用

## Spec

### PMSpec

規格由使用者口述，以下即為本 task 的規格。

- 編輯行為要記進使用者歷程，`Ctrl+Z` 復原、`Ctrl+Y` 還原復原。
- 歷程深度上限 20 筆，滿了丟掉最舊的一筆。
- 納入歷程的範圍共三類：(1) 上方查詢資料與下拉選單 (2) 下方的逐列編輯與下方的詳細編輯 (3) 設定的編輯。
- 設定視窗的開啟與關閉不納入歷程，設定的 tab 切換不納入歷程。
- 刪除不納入歷程。刪除的語意就是確定不要了，不需要救回。
- 使用者想「刪掉」一筆工作單時，通常是加一個 state 來標記（closed、remove 之類），而不是真的刪除。這是多數 scrum ticket 工具的常見邏輯。
- 工時統計不因 state 過濾。要讓一筆不計入工時，由使用者自己把工時改成 0。
- 本功能的目的是為將來的**多筆更新**鋪路，屆時要能用一次 `Ctrl+Z` 還原整批修改。**多筆更新目前還沒有這個行為，它是本 task 的前提，不是本 task 的工作範圍**；本 task 只負責讓歷程項的結構容納得下「一步包含多筆變更」。

### HistorySpec

歷程的對外行為：

- `Ctrl+Z` 復原上一步，`Ctrl+Y` 還原復原。主視窗與設定視窗按下都有效，共用同一條歷程。
- 焦點在輸入框或文字區域時，`Ctrl+Z` 歸該欄位的原生還原，不走歷程；離開欄位後才由歷程接手。
- 查詢條件的一連串調整算一步：連續調整篩選後按一次 `Ctrl+Z`，整組條件回到調整前的狀態。
- 新增紀錄可以復原：`Ctrl+Z` 移除剛建的那筆，`Ctrl+Y` 重新建立。
- 刪除不可復原，且刪除發生時整條歷程清空。
- 歷程不跨 session：關掉 app 再開，歷程是空的。
- 歷程達 20 筆後最舊的一筆被丟棄，不再能復原。
- 復原一筆工作紀錄前若偵測到它已被其他 app 實例改過，放棄該筆復原並提示；設定、狀態、假日、請假的復原直接覆蓋。
- 歷程為空或已到底時，按 `Ctrl+Z`／`Ctrl+Y` 沒有作用。

## Scope

- 主視窗與設定視窗的編輯行為納入可復原的使用者歷程，以 `Ctrl+Z`／`Ctrl+Y` 操作。
- 不含：設定視窗的開啟與關閉。
- 不含：設定的 tab 切換。
- 不含：純檢視操作（選取某一列、展開詳情、捲動、分隔線拖曳與欄寬拖曳的視覺狀態）。
- 不含：資料庫路徑切換（`change_db_path`），它會整個換掉資料來源。
- 不含：刪除的復原，工作紀錄、使用者、狀態、假日、請假皆同。刪除的語意是確定不要了，按 `Ctrl+Z` 不會救回；刪除發生時直接清空歷程。
- 不含：多筆更新本身。它目前還沒有這個行為，是 006 的前提而非工作範圍；006 只負責讓歷程項的結構容納得下它。
- 不含：歷程跨 session 保留。app 關掉歷程即消失，不做持久化。

## Survey 現況調查

### 編輯入口盤點 — 上方查詢資料與下拉選單

- 關鍵字 `#keyword`：`input` 事件搭 200ms debounce 觸發 `refresh()`（`src/app.js:859`）。
- 範圍下拉 `#rangeScope`：`change` 觸發 `applyScope()`，回填起訖日（`src/app.js:861`）。
- 起訖日 `#dateFrom`／`#dateTo`：`change` 時把 scope 切成 `custom` 再 `refresh()`（`src/app.js:867`）。
- 狀態過濾：對話框內每個 checkbox 的 `change` 直接改 `state.statusFilter` 並 `refresh()`（`src/app.js:141`）。
- 使用者過濾 `#userFilter`：`change` 觸發 `refresh()`（`src/app.js:877`）。
- 排序：欄位標題按鈕的 `click` 改 `state.sortField`／`sortDir`（`src/app.js:313`）。
- 提醒範圍 `#hoursBadge`：`change` 寫入 `reminder_range` 設定（`src/app.js:879`）。這個控制項在主視窗工具列，與設定頁的「提醒範圍」是同一個 setting。
- 以上七項的狀態每次 `refresh()` 結束都整包寫進 `last_filter` 設定（`src/app.js:817` 的 `rememberFilter`）。因此查詢條件已經有一份可序列化的完整快照格式，歷程可以直接沿用它。

### 編輯入口盤點 — 逐列編輯與詳細編輯

- 逐列編輯的六個欄位（工單、標題、工時、狀態、日期、使用者）：每個都綁 `change` → `saveSelected()`（`src/app.js:199`）。
- 清單列上的狀態下拉（非編輯狀態也能直接切）：`change` → `API.setEntryStatus`（`src/app.js:220`）。
- 詳情的內容 textarea `#fContent`：`change` → `saveSelected()`（`src/app.js:889`）。
- 新增紀錄：加號按鈕與 `Ctrl+N` 都走 `createEntry()` → `API.saveEntry`（`src/app.js:370`、`src/app.js:906`）。
- 刪除紀錄：編輯列的 `✕` → `removeEntry()`，有 `window.confirm` 確認（`src/app.js:184`、`src/app.js:530`）。
- 關聯工單：加入走 `API.linkEntries`（`src/app.js:749`），移出走 `API.unlinkEntry`（`src/app.js:399`）。
- 欄寬拖曳與清單寬度拖曳：寫進 `window_layout` 設定（`src/app.js:260`、`src/app.js:801`）。屬於視覺狀態。

### 編輯入口盤點 — 設定的編輯

- 一般：語言、主題、間距、提醒範圍、預設狀態，各自 `setSetting`（`src/settings.js:355`、`:360`、`:365`、`:370`、`:385`）。
- 使用者：新增 `createUser`（`:379`）、改名與改工時 `updateUser`（`:89`、`:100`）、刪除 `deleteUser`（`:109`）。
- 狀態：新增 `createStatus`（`:394`）、改名與改色 `updateStatus`（`:157`、`:166`）、刪除 `deleteStatus`（`:402`）。
- 假日：新增 `saveHoliday`（`:414`）、刪除 `deleteHoliday`（`:228`）、一鍵匯入 `importHolidays`（`:277`）。
- 請假：新增與修改 `saveLeave`（`:430`）、刪除 `deleteLeave`（`:315`）。
- sprint 基準：編號、起始日、天數三個 `setSetting`（`:442`～`:444`）。
- 資料庫路徑：`changeDbPath`（`:450`）。

### 架構現況與難點

- **設定是獨立的 Tauri window**：label `settings`、載入 `settings.html`（`src-tauri/src/commands.rs:292`～`:313`）。主視窗與設定視窗是兩個 webview、兩份獨立的 JS context，彼此只透過資料庫與 `window` 的 `focus` 事件（`src/app.js:904` 的 `reloadSettings`）間接同步。**只存在前端記憶體的歷程無法跨這兩個視窗**。
- **沒有任何既有的 undo 基礎設施**：`state` 只保存當前值，沒有前後值的記錄；Rust 端沒有歷程表，schema 的六張表都沒有版本或軟刪欄位（`src-tauri/src/db.rs:9`～`:57`）。
- **刪除全都是硬刪**：`delete_entry` 直接 `DELETE FROM entries`，並且會連帶 `dissolve_if_alone` 動到群組（`src-tauri/src/store.rs:272`）。要復原就得在刪除前自己留快照。
- **entries 的 id 是 AUTOINCREMENT**：重建一筆被刪的紀錄會拿到新 id，所以 `state.selectedId`、`group_id` 關聯、以及 redo 時要指向哪一筆，都不能靠原本的 id。
- **刪除狀態有連帶改動**：`delete_status` 先把所有掛該狀態的 entries 改掛 replacement，再刪掉狀態（`src-tauri/src/store.rs:162`～`:179`）。復原需要記下「原本有哪些 entries 掛在這個狀態」，單純重建狀態列不足以還原。
- **刪除使用者會清空擁有者**：schema 的 `entries.user_id` 是 `ON DELETE SET NULL`（`src-tauri/src/db.rs:28`），所以刪使用者會把那些紀錄變成無擁有者，復原同樣要記下受影響的紀錄。
- **一鍵匯入假日是整年覆蓋**：前端以 `replace = true` 呼叫，後端先 `clear_holidays_in_year` 再逐筆 upsert（`src-tauri/src/store.rs:424`、`:440`）。復原要先備份那一年的全部資料。
- **查詢條件與視覺狀態都已可序列化**：`last_filter`、`window_layout` 都是 JSON 字串存在 `app_settings`，歷程要記錄它們成本很低。
- **輸入框有瀏覽器原生的 undo**：WebView2 對 `input`／`textarea` 本來就支援 `Ctrl+Z` 還原輸入中的文字。全域歷程若攔下 `Ctrl+Z`，會蓋掉使用者在輸入框內的原生還原行為；不攔則焦點在輸入框時全域歷程失效。
- **既有 keydown 只處理 `Ctrl+N`**：`document` 層的 keydown 監聽，`event.ctrlKey && event.key === 'n'` 時 `preventDefault` 並新增紀錄（`src/app.js:906`）。新的快捷鍵要加在同一處。
- **主視窗是常駐的**：`CloseRequested` 被 `prevent_close()` 攔下、只做 `hide()`（`src-tauri/src/main.rs:108`～`:115`），所以主視窗的 JS context 活在整個 app 的生命週期內，設定視窗開了又關都不影響它。
- **前端可以直接跨視窗傳訊**：`withGlobalTauri` 為 `true`（`src-tauri/tauri.conf.json:10`）且是 Tauri v2（`src-tauri/Cargo.toml:12`），所以 `window.__TAURI__.event` 的 `emit`／`listen` 可用。目前 `api.js` 只用到 `core.invoke`（`src/api.js:1`），event 尚未使用。
- **設定視窗有現成的重載入口**：`reload()`（`src/settings.js:460`）。主視窗目前靠 `window` 的 `focus` 事件呼叫 `reloadSettings()` 單向同步（`src/app.js:904`）。
- **編輯是 `change` 事件觸發、不是 `input`**：所以每一筆歷程天然對應「離開欄位時的一次完整修改」，不會出現每打一個字一筆的問題。

### 刪除機制現況：全部硬刪除

- 六張表都沒有軟刪欄位（`is_deleted`、`deleted_at` 之類），schema 見 `src-tauri/src/db.rs:9`～`:57`。
- `delete_entry`、`delete_user`、`delete_holiday`、`delete_leave` 都是直接 `DELETE`（`src-tauri/src/store.rs:272`、`:121`、`:448`、`:500`）；`delete_status` 先把掛該狀態的紀錄改掛 replacement 再 `DELETE`（`:162`）。
- `users.is_active` 看起來像軟刪，其實不是：它由 `update_user` 維護（`src-tauri/src/store.rs:115`）代表「停用」，而 `delete_user` 是真刪。而且前端目前沒有任何 UI 在切換它，`settings.js:89`、`:100` 都只是把原值傳回去。
- 規格的「加 state 標記而非刪除」在現有機制內就做得到，不必改 schema：`statuses` 是使用者可自訂的表（`create_status`），而狀態變更走 `API.setEntryStatus`／`saveEntry` 這條值修改的路徑（`src/app.js:220`、`:199`），本來就落在歷程範圍內。
- 規格的「工時統計不因 state 過濾」與程式現況一致：`filled` 的查詢是 `SELECT work_date, SUM(hours) FROM entries WHERE user_id = ? AND work_date BETWEEN ? AND ?`，不帶任何狀態條件（`src-tauri/src/store.rs:672`），與 task 002 定案的「全部狀態都計入已填時數」同一個設計。不要當成缺陷去修。

### 空結果無法新增工單的兩個根因

- 加號按鈕的 DOM 一直都在：`renderList()` 的空清單分支本來就 append 了 `buildAddButton()`（`src/app.js:353`）。問題是看不到，而不是沒有。
- `.empty` 設了 `height: 100%`（`src/styles.css:656`），而 `.list` 是 `flex: 1` 搭 `overflow-y: auto`。空狀態因此吃掉整個可視高度，把後面的 `.add-row` 推到捲動範圍之外。
- `.empty` 同時用在右側詳情的 `#detailEmpty`（`src/index.html:40`），那裡需要撐滿，所以不能直接改 `.empty` 本身。
- 第二個根因與可見性無關：關鍵字搜尋狀態下新增的工單標題是空的、不符合關鍵字，`refresh(id)` 回來的結果不含它。於是 `state.selectedId` 指向新工單、右側詳情表單打開，左側卻沒有對應的列，`scrollToSelected()` 也找不到目標，看起來像按鈕沒反應。這個行為在 task 005 之前就存在。
- `navigateTo()` 早有「目標不在當前結果時放寬過濾條件並提示」的既有慣例，連 i18n 文案 `toast.filterRelaxed`（「已放寬過濾條件以顯示該筆」）都已存在（`src/app.js:558`、`src/i18n.js`）。

## Design 設計

### 歷程本體

1. 一筆歷程項是「一組變更」，每個變更記下種類、目標識別、`before` 與 `after`。undo 對每個變更套用 `before`，redo 套用 `after`。
2. 套用一律呼叫既有的寫入 API，不新增 Rust 指令。
3. 歷程由主視窗持有。設定視窗用 `window.__TAURI__.event` 把變更項 emit 給主視窗 push，也用 event 請求 undo／redo；主視窗套用完 emit 回去，設定視窗跑它既有的 `reload()`。
4. `Ctrl+Z`／`Ctrl+Y` 綁在 `document` 的 keydown，與既有 `Ctrl+N` 共存；用 `event.target` 是否為 `input`／`textarea` 決定要不要攔截。
5. 查詢條件的合併靠比對前後的 `last_filter` 快照，相同就不 push。
6. `entries` 的變更在 push 時一併記下 `updated_at`，套用前比對。

> 設計決策（已定案）：歷程存在主視窗的前端，不新增 Rust 歷程結構。理由：刪除不納入後歷程項只剩舊值→新值，undo 等於帶舊值重發既有寫入 API；主視窗常駐（`src-tauri/src/main.rs:108`～`:115` 攔下 `CloseRequested` 只做 `hide()`），可安全持有唯一歷程；跨視窗用 Tauri v2 的 event 即可（`withGlobalTauri` 為 `true`），不必動 Rust。代價是歷程不跨 session，而那正是另一條定案。

> 設計決策（已定案）：undo／redo 一律走既有的寫入 API。理由：既有的後端規則照樣生效，例如 `set_setting` 對 sprint 基準空值的回填（task 004 定案）不會被繞過。

> 設計決策（已定案）：查詢條件與資料編輯共用同一條歷程，但查詢條件的連續變更合併成一筆。理由：`last_filter` 已有完整快照格式，合併只需比對前後快照；兩條歷程得在畫面上交代「現在復原哪一條」，而工具列沒有位置放這種提示。

> 設計決策（已定案）：焦點在 `input`／`textarea` 時不攔 `Ctrl+Z`。理由：編輯是 `change` 事件才寫入，打字到一半的內容還沒進歷程，攔下來會把未存的輸入整段丟掉。

> 設計決策（已定案）：歷程不跨 session，app 關掉即消失。理由：編輯集中在當下補工時，隔天復原昨天的編輯幾乎都是誤操作；也讓併發衝突只剩同一 session 內多實例這個罕見情境。

> 設計決策（已定案）：新增紀錄納入歷程。理由：誤按加號是常見小失誤；剛建的紀錄沒有群組關聯也沒有內容，redo 重建換 id 不影響任何東西。

> 設計決策（已定案）：設定類的刪除（使用者、狀態、假日、請假）也不納入歷程，刪除時清空歷程。理由：刪狀態會把掛它的紀錄改掛 replacement、刪使用者會把紀錄的擁有者清空（`src-tauri/src/store.rs:162`、`src-tauri/src/db.rs:28`），`createStatus`／`createUser` 還原不了這些連帶改動，納入只會造成「看起來復原了但資料不一致」。與工作紀錄的刪除同一個處理。

> 設計決策（已定案）：新增請假後以 `listLeaves` 回查 id，取同使用者同日期中最大的那個。理由：`save_leave` 回傳 `Answer<()>` 不給 id（`src-tauri/src/commands.rs:231`），而定案是不動 Rust。

> 設計決策（已定案）：只有 `entries` 做併發衝突偵測（比對 `updated_at`），其他表直接覆蓋。理由：`entries` 已有 `updated_at` 與索引（`src-tauri/src/db.rs:38`、`:63`），成本幾乎為零；另外四張表沒有時間欄位，要同等保護得先加 migration，而它們被兩個實例同時修改的機率遠低於工作紀錄。

### 空結果也能新增工單

1. 清單空狀態改用修飾 class `.empty.list-empty` 把 `height` 覆蓋成 `auto`，`min-height: 120px` 保留，所以空狀態仍有留白、加號緊接在下方且都在可視範圍內。
2. `createEntry()` 新增後先 `refresh(id)`，若結果不含新工單才放寬過濾條件、再 `refresh(id)` 並 toast 提示。

> 設計決策（已定案）：用修飾 class 覆蓋 `height`，不改 `.empty` 本身、也不把 `.list` 改成 flex container。理由：`.empty` 由詳情面板共用且那裡需要撐滿；把 `.list` 改成 flex 會動到所有列的排版，改動面遠大於需求。

> 設計決策（已定案）：新增後看不到就放寬過濾條件並提示，沿用 `navigateTo()` 的既有行為與 `toast.filterRelaxed` 文案。理由：repo 已有這套互動語言，不另外發明「新增時自動把關鍵字當標題」或「停留在空清單」等新行為。

> 設計決策（已定案）：把放寬過濾條件的四行抽成 `relaxFilters()`。理由：`navigateTo()` 與 `createEntry()` 兩處需要同一段邏輯，純提取、`navigateTo()` 行為不變。

> 設計決策（已定案）：刪除不納入歷程，刪除發生時清空歷程。理由：刪除的意圖明確、不需要救回；留著指向已刪資料的歷程項只會造成錯亂。使用者工作流中的「刪掉」多半是改 state 標記，那條路徑走的是值的修改，本來就在歷程範圍內。

> 設計決策（已定案）：歷程項的結構一開始就設計成容納「一組變更」（一筆或多筆），而不是單一欄位的舊值→新值。理由：多筆更新是這個功能的前提目的，雖然不在 006 的工作範圍，但結構若只支援單筆，屆時得整個改寫。

## Wait Decision

