# Task 003 Leave Hours Default

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

### goal 請假時數預設值

- [ ] 設定面「假日與請假」新增請假時，時數預設為該使用者設定的每日必填工時
- [ ] 使用者沒有設定每日必填工時時，時數預設為 8

### task 執行（請假時數預設值）

- [x] `settings.html`：拿掉 `#newLeaveHours` 寫死的 `value="8"`
- [x] `settings.js`：新增 `syncLeaveHours()`，依 `#newLeaveUser` 選中的使用者回填 `#newLeaveHours`
- [x] `settings.js`：`renderUsers()` 填完 `#newLeaveUser` 選項後呼叫 `syncLeaveHours()`
- [x] `settings.js`：`#newLeaveUser` 的 `change` 事件呼叫 `syncLeaveHours()`
- [x] `settings.js`：`syncLeaveHours()` 中 `daily_required_hours` 大於 0 才採用，否則回填 `8`
- [x] `settings.js`：使用者分頁修改每日必填工時，輸入 0 或清空時存成 `8`

### 阻塞事項

- [ ] 目前無阻塞事項

## Spec

### PMSpec

- 本 task 無 PM 規格書，需求來源為使用者口述：使用者會設定每日必填工時；在假日與請假新增請假時，輸入的時數預設不可以是 8，要是該使用者設定的必填工時，除非使用者沒有設定才會是 8。

### UiSpec

#### 設定面「假日與請假」新增請假列

| 情況 | `#newLeaveHours` 預設值 |
|---|---|
| 選中的使用者每日必填工時為 7.5 | `7.5` |
| 選中的使用者每日必填工時為 0（視為沒有設定） | `8` |
| 尚無任何使用者 | `8` |

- 切換 `#newLeaveUser` 時，`#newLeaveHours` 立即改成新使用者的預設值。
- 預設值可手動修改，送出時以輸入框的值為準。

## Scope

- 設定面新增請假列的時數預設值，改為跟隨選中使用者的 `daily_required_hours`。
- 使用者分頁修改每日必填工時時，0 或空值改存 `8`。
- 不含：Rust 端與 schema 改動，`save_leave` 仍接收前端傳入的時數。
- 不含：既有請假紀錄的時數，不回頭改寫。
- 不含：請假時數上限檢查（例如不得超過每日必填工時）。

## Survey 現況調查

- 新增請假列在 `src/settings.html:91`～`94`：使用者下拉 `#newLeaveUser`、日期 `#newLeaveDate`、時數 `#newLeaveHours`、按鈕 `#addLeave`。
- `#newLeaveHours` 在 HTML 寫死 `value="8"`（`src/settings.html:93`），切換使用者不會變動。
- `#addLeave` 送出時取 `Number(dom.newLeaveHours.value) || 0`，送完不清空輸入框（`src/settings.js:416`）。
- `#newLeaveUser` 的選項在 `renderUsers()` 尾端以 `fillOptions` 重填，每次 `reload()` 都會跑（`src/settings.js:119`）。
- 使用者資料在 `state.users`，每筆帶 `daily_required_hours`（`src/settings.js:98`、`src-tauri/src/models.rs:7`）。
- `users.daily_required_hours` 為 `REAL NOT NULL DEFAULT 8`（`src-tauri/src/db.rs:12`），資料庫層不會出現 NULL。
- 新增使用者時輸入框留空會帶 8（`src/settings.js:372`）；修改既有使用者時輸入框清空會存成 0（`src/settings.js:100`）。因此「沒有設定」在資料上只可能表現為 0。
- 缺口計算對 0 的處理：`hours_report` 直接拿 0 當每日必填，應填時數為 0（`src-tauri/src/store.rs:593`）。

## Design 設計

1. 預設值只在前端決定：`syncLeaveHours()` 從 `state.users` 找出 `#newLeaveUser` 選中的使用者，取 `daily_required_hours` 回填。
2. 回填時機：`renderUsers()` 重填下拉後、`#newLeaveUser` 切換時。修改使用者的每日必填工時會觸發 `reload()`，新值隨之帶入。
3. `daily_required_hours` 大於 0 視為有設定，回填設定值；等於 0 或找不到使用者視為沒有設定，回填 `8`。

> 設計決策（已定案）：每日必填工時 0 不是合法設定值，一律視為沒有設定。理由：資料層欄位為 `NOT NULL DEFAULT 8`，0 只會出現在使用者分頁清空欄位時，代表沒設定。

> 設計決策（已定案）：使用者分頁輸入 0 或清空時存成 8。理由：0 不是合法值就不寫進資料庫，請假預設與缺口計算對每日必填工時的解讀才一致；新增使用者的預設同樣是 8。

### 資料流

```text
[設定面 使用者分頁 每日必填工時]
   │  updateUser → reload() → renderUsers()
   ▼
[#newLeaveUser 重填選項] ──change──┐
   │                                │
   ▼                                ▼
syncLeaveHours()  ──▶  [#newLeaveHours] = daily_required_hours（為 0 時 8）
   ▼
[#addLeave] → API.saveLeave(..., hours)
```

## Wait Decision

目前無待確認事項。
