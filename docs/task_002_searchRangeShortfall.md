# Task 002 Search Range Shortfall Badge

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

### goal 搜尋區間缺口提示

- [ ] 在既有缺口徽章（`XXX 尚缺 xxx 小時`）的左邊，新增一個同類的缺口提示
- [ ] 新提示顯示為 `總計 XX 小時 尚缺 XX 小時`
- [ ] 新提示的檢查範圍使用搜尋列當前的日期區間

### task 執行（搜尋區間缺口提示）

- [x] `index.html`：在 `#hoursBadge` 左邊加 `<span class="badge static" id="rangeShortBadge" hidden>`
- [x] `styles.css`：加 `.badge.static`，拿掉下拉箭頭與右側 padding、游標改預設
- [x] `i18n.js`：兩份字典各加 `hours.rangeShort`（`總計 {total} 小時 尚缺 {n} 小時`／`{total}h total · {n}h short`）
- [x] `app.js`：`dom` 清單加 `rangeShortBadge`
- [x] `app.js`：新增 `refreshRangeBadge()`，隱藏條件成立時直接 `hidden = true` 並 return
- [x] `app.js`：`refreshRangeBadge()` 以 `API.hoursReport(selectedUserId(), 'custom', dateFrom, dateTo)` 取缺口
- [x] `app.js`：`refreshBadge()` 開頭呼叫 `refreshRangeBadge()`
- [x] `app.js`：`diff < 0` 顯示尚缺套 `.short`，`diff = 0` 顯示已達標套 `.exact`，`diff > 0` 顯示超出套 `.over`
- [x] `i18n.js`：兩份字典各加 `hours.rangeMet`（`總計 {total} 小時，已達標`／`{total}h total · on target`）
- [x] `i18n.js`：兩份字典各加 `hours.rangeOver`（`總計 {total} 小時 超出 {n} 小時`／`{total}h total · {n}h over`）

### 阻塞事項

- [ ] 目前無阻塞事項

## Spec

### PMSpec

- 本 task 無 PM 規格書，需求來源為使用者口述：在 `XXX 尚缺 xxx 小時` 的左邊新增一個類似功能的 error，顯示為 `尚缺 XXX 小時`，檢查範圍使用當前搜尋的範圍。

### UiSpec

#### 搜尋區間缺口提示 — `#rangeShortBadge`

顯示條件全部成立時才出現，任一不成立即隱藏：

- 使用者下拉選定單一使用者（未選、無擁有者皆隱藏）。
- 日期起與日期訖皆有值。
- 關鍵字為空。
- 狀態過濾全不選。

顯示內容：

- 已填少於應填：`總計 30 小時 尚缺 10 小時`，琥珀色（`.badge.short`）。`總計` 取 `report.filled`、`尚缺` 取 `-report.diff`。
- 已填等於應填：`總計 40 小時，已達標`，綠色（`.badge.exact`）。
- 已填多於應填：`總計 43 小時 超出 3 小時`，藍色（`.badge.over`）。
- English：`30h total · 10h short`／`40h total · on target`／`43h total · 3h over`。
- 位置：搜尋列 `#hoursBadge` 左邊。
- 形式：唯讀標籤。

## Scope

- 在主畫面搜尋列新增搜尋區間缺口提示，範圍取搜尋列的日期起訖。
- 不含：Rust 端改動，`hours_report` 既有的 `custom` 路徑已支援任意起訖。
- 不含：系統匣 tooltip，系統匣維持跟隨 `reminder_range`。
- 不含：既有 `#hoursBadge` 的行為與文案。

## Survey 現況調查

- 既有缺口徽章是 `<select class="badge" id="hoursBadge">`，位於搜尋列 spacer 右側、設定鈕左側（`src/index.html:20`）。
- `refreshBadge()` 對今日、本週、本 sprint 三個範圍各呼叫一次 `API.hoursReport`，組成下拉選項，並把選中範圍的文字送進系統匣（`src/app.js:576`）。
- `badgeText()` 依 `report.diff` 產出尚缺、剛好達標、超出三種文字與色調（`src/app.js:547`）。
- `hours_report` 先用 `resolve_range` 解析 scope；scope 為 `custom` 時改用傳入的 `custom_from`／`custom_to`（`src-tauri/src/store.rs:557`）。
- `custom_from`／`custom_to` 任一端解析失敗時，後端回退成本週區間（`src-tauri/src/store.rs:573`）。搜尋列允許日期起訖留空，此時直接呼叫會拿到本週的缺口。
- 已填時數計入該使用者區間內全部紀錄，關鍵字與狀態過濾不影響（`src-tauri/src/store.rs:636`）。
- 過濾狀態的取值：關鍵字 `dom.keyword.value`、狀態複選 `state.statusFilter`（`src/app.js:45`）。
- `selectedUserId()` 在未選（`any`）與無擁有者（`unowned`）時回 `null`，`hours_report` 收到 `null` 回 `has_user: false`（`src/app.js:40`、`src-tauri/src/store.rs:580`）。
- `refreshBadge()` 由 `refresh()` 與提醒範圍切換呼叫（`src/app.js:628`、`src/app.js:805`），搜尋條件或使用者變動都會觸發。
- `.badge.short`／`.exact`／`.over` 三種色調定義在 `src/styles.css:295`；`.badge` 本身帶下拉箭頭背景圖（`src/styles.css:282`）。

## Design 設計

1. 前端以 `API.hoursReport(selectedUserId(), 'custom', dom.dateFrom.value, dom.dateTo.value)` 取缺口，後端零改動。
2. 新提示用 `<span>` 搭 `.badge.static`，色調沿用 `.badge.short`／`.badge.exact`／`.badge.over`，與右側徽章同一族。
3. 重算掛在 `refreshBadge()` 開頭，跟著既有徽章同步更新；放開頭是因為 `refreshBadge()` 在三個範圍都取不到結果時會提前 return。
4. 文案不帶範圍字樣，範圍由左側日期起訖自明。
5. 隱藏條件先在前端判斷：未選單一使用者、日期缺一端、有關鍵字、有狀態過濾，任一成立就不呼叫 `hours_report`。
6. `diff < 0` 顯示尚缺並套 `.short`；`diff = 0` 顯示已達標並套 `.exact`；`diff > 0` 顯示超出並套 `.over`。

> 設計決策（已定案）：一律以 `custom` 帶搜尋列起訖呼叫 `hours_report`。理由：今日、本週、本月、sprint 等快捷都已回填起訖日期，單一路徑即涵蓋全部快捷，計算口徑與既有徽章一致。

> 設計決策（已定案）：已填時數只取日期與使用者，有關鍵字或狀態過濾時整個提示隱藏。理由：與 task 001 定案的「全部狀態都計入已填時數」一致；過濾中的列表只是部分紀錄，此時顯示的總計會與列表對不上，乾脆不顯示。

> 設計決策（已定案）：`總計` 取已填時數 `report.filled`。理由：`總計` 加 `尚缺` 即為應填時數，一眼看出區間的完整需求。

> 設計決策（已定案）：三種結果三種樣式，不足琥珀、剛好達標綠、超出藍。理由：與 task 001 缺口徽章的色調定案一致，右側 `#hoursBadge` 同樣分三種。

> 設計決策（已定案）：日期起或日期訖留空時隱藏。理由：後端遇空日期會回退成本週，顯示出來會被誤認為搜尋區間的缺口。

### 資料流

```text
[搜尋列 日期起訖 / 使用者]
   │  refresh() → refreshBadge()
   ▼
refreshRangeBadge()
   │  API.hoursReport(userId, 'custom', dateFrom, dateTo)
   ▼
store::hours_report → calendar::summarize
   ▼
[#rangeShortBadge]  diff < 0 → 總計 filled 小時 尚缺 -diff 小時
```

## Wait Decision

目前無待確認事項。
