# Task 005 Global Keyword Search

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

### goal 關鍵字搜尋搜全部

- [ ] 關鍵字非空時，搜尋標題、內容、工單的範圍是全部紀錄
- [ ] 搜尋結果不受時間區間 filter 影響
- [ ] 搜尋結果不受自訂時間起訖影響
- [ ] 搜尋結果不受狀態 filter 影響
- [ ] 搜尋結果不受使用者 filter 影響

### task 執行（關鍵字搜尋搜全部）

- [x] `app.js`：`currentQuery()` 在關鍵字非空時，日期起訖送空字串、狀態送空陣列、使用者送 `any`
- [x] `app.js`：關鍵字為空時維持原本行為，四個 filter 照常套用
- [x] `app.js`：兩條路徑都保留使用者選的排序欄位與方向
- [x] `query.rs`：確認既有測試已覆蓋兩種查詢形狀，不加重複測試
- [x] 以 node 腳本驗證 `currentQuery()` 兩條路徑的輸出

## Spec

### PMSpec

- 本 task 無 PM 規格書，需求來源為使用者口述：搜尋標題、內容、工單時，搜尋的內容是全部，不受右方的 filter 影響；補充界定為「不受時間區間 filter 影響、不受自訂時間起訖影響、不受狀態 filter 影響、不受使用者 filter 影響，是真實意義的不受所有 filter 影響」。

### QuerySpec

`currentQuery()` 送給 `search_entries` 的欄位值：

| 欄位 | 關鍵字為空 | 關鍵字非空 |
|---|---|---|
| `keyword` | 輸入框內容（空字串） | 輸入框內容 |
| `date_from` | `#dateFrom` 的值 | `''` |
| `date_to` | `#dateTo` 的值 | `''` |
| `status_ids` | `state.statusFilter` 的內容 | `[]` |
| `user_filter` | `any`／`unowned`／`one` | `any` |
| `user_id` | 選中的使用者 id 或 `null` | `null` |
| `sort_field` | `state.sortField` | `state.sortField` |
| `sort_dir` | `state.sortDir` | `state.sortDir` |

## Scope

- 主畫面關鍵字搜尋改為忽略四個 filter，查詢全部紀錄。
- 不含：Rust 端改動，`build_filter` 既有的「值為空就不加條件」已足以表達無 filter 的查詢。
- 不含：關聯工單選擇器的搜尋（`link_candidates` 自有範圍與 `LIMIT 50`）。
- 不含：filter 控制項在搜尋期間的視覺變化，控制項維持可操作、值原樣保留。
- 不含：新增紀錄時的工作日期與擁有者取值，該議題在 `task_004` 的 Wait Decision 追蹤。

## Survey 現況調查

- `currentQuery()` 把關鍵字與四個 filter 一起組成 `EntryQuery` 送出（`src/app.js:58`）。
- `build_filter` 逐項檢查後才加條件：關鍵字 trim 後非空才加三個 `LIKE`（`src-tauri/src/query.rs:44`）、`date_from` 非空才加 `>=`（`:56`）、`date_to` 非空才加 `<=`（`:62`）、`status_ids` 非空才加 `IN`（`:68`）、`UserFilter::Any` 不加條件（`:84`）。因此「無 filter」在後端只要送空值即可表達，不需要新參數或新旗標。
- `clauses` 全空時 `where_sql` 為空字串，查詢等同全表（`src-tauri/src/query.rs:88`）。
- 關鍵字搜尋涵蓋標題、內容、工單三欄，`content` 以 `IFNULL` 處理 NULL，並以 `ESCAPE '\'` 搭 `escape_like` 轉義 `%`、`_`、`\`（`src-tauri/src/query.rs:46`、`:97`）。
- 搜尋框是 `input` 事件搭 200ms debounce 觸發 `refresh()`（`src/app.js:835`）。
- 排序與 filter 是兩套獨立機制，`build_order` 只看 `sort_field`／`sort_dir`（`src-tauri/src/query.rs:26`）。
- `search_entries` 沒有 `LIMIT`，回傳全部符合的列（`src-tauri/src/store.rs:206`）。本工具是單機 SQLite 桌面程式，且既有組合「全部使用者＋不限日期」早已會回全表，忽略 filter 不引入新的量級。
- `#rangeShortBadge` 在關鍵字非空時隱藏（`src/app.js:594`）。搜尋狀態下不顯示區間時數的慣例已經存在，本次改動與它方向一致。
- `#hoursBadge` 自己帶 scope 呼叫 `hours_report`，與清單查詢無關（`src/app.js:625`），搜尋期間照常顯示今日／本週／本 sprint 的進度。
- `navigateTo()` 在目標紀錄落在當前清單外時，清空關鍵字與狀態過濾再重查（`src/app.js:541`），已有「放寬條件以顯示目標」的既有行為。
- `build_filter` 的兩種目標形狀既有測試已覆蓋：`keyword_hits_title_content_and_ticket` 驗關鍵字搭空白 filter 只組出三個 `LIKE`（`src-tauri/src/query.rs:238`），`conditions_are_joined_with_and` 驗四個 filter 齊備時組出四個 `AND` 與七個參數（`src-tauri/src/query.rs:318`）。本次後端零改動，不另加重複測試。
- repo 只有 Rust 測試，前端沒有測試框架（`src/` 下無 test runner 或 spec 檔），前端邏輯以 node 腳本單獨驗證查詢物件的形狀。
- 四個 filter 的 UI 值在搜尋期間原樣保留在 DOM 上，關鍵字清空後 `currentQuery()` 自然回到套用 filter 的那條路徑，不需要另外存檔或還原。

## Design 設計

1. 判斷點放在 `currentQuery()`：關鍵字 trim 後非空就回傳「只帶關鍵字與排序」的查詢物件，其餘欄位送空值。後端零改動。
2. 判斷用 `trim()`，純空白的輸入視為沒有搜尋，仍套用 filter。
3. 排序不屬於 filter，兩條路徑都帶使用者選的 `sort_field`／`sort_dir`。
4. filter 控制項維持可操作、值不動。搜尋期間調整 filter 不影響結果，關鍵字清空後立刻生效。

> 設計決策（已定案）：以「送空值」表達無 filter，不新增 `ignore_filters` 之類的參數。理由：`build_filter` 既有語意已是「值為空就不加條件」，送空值即精準表達需求，`models.rs` 的 `EntryQuery` 與 Rust 端都不必改。

> 設計決策（已定案）：關鍵字清空後回到套用 filter 的狀態，靠 filter 控制項的值始終保留達成，不另存快照。理由：四個 filter 的值在搜尋期間完整留在 DOM，`currentQuery()` 每次重新讀取，沒有需要還原的狀態。

> 設計決策（已定案）：搜尋期間不改變 filter 控制項的外觀、不加提示文字。理由：`#rangeShortBadge` 在關鍵字非空時隱藏的既有慣例已經傳達「搜尋狀態下不看區間」，維持最小改動。

### 資料流

```text
[#keyword 輸入]
   │  input + 200ms debounce → refresh()
   ▼
currentQuery()
   │  keyword.trim() 非空 → { keyword, 日期空, 狀態空, user_filter: any, 排序 }
   │  keyword.trim() 為空 → { keyword, 日期, 狀態, 使用者, 排序 }
   ▼
API.searchEntries → store::search_entries → query::build_filter
   │  非空才加條件，全空則無 WHERE
   ▼
[清單顯示全部符合關鍵字的紀錄]
```

## Wait Decision

