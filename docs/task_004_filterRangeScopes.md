# Task 004 Filter Range Scopes

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

### goal 篩選範圍擴充

- [ ] 篩選條件提供十個預設範圍：今日、上週、本週、下週、上個 sprint、本 sprint、下個 sprint、上月、本月、下月
- [ ] 自訂區間保留，手改起訖日自動切換為自訂
- [ ] sprint 基準永遠有值，不存在「尚未設定 sprint 基準」的情境

### task 執行（篩選範圍擴充）

- [x] `store.rs`：`resolve_range` 的 week 分支擴成 `last_week | week | next_week`
- [x] `store.rs`：`resolve_range` 的 month 分支擴成 `last_month | month | next_month`
- [x] `store.rs`：`resolve_range` 的 sprint 分支擴成 `sprint | last_sprint | next_sprint`
- [x] `store.rs`：補回被誤刪的 `"today"` 分支
- [x] `i18n.js`：兩份字典各加 `range.last_week`／`next_week`／`next_sprint`／`last_month`／`next_month`
- [x] `i18n.js`：修掉重複兩次的 `range.today`
- [x] `app.js`：`SCOPES` 改為十個預設範圍加 `custom`，順序即下拉顯示順序
- [x] `app.js`：拆出 `SPRINT_SCOPES`，讓 `badgeText` 的 sprint 判斷涵蓋 `next_sprint`
- [x] `index.html`：還原兩個日期欄位（移除中途被加上的 `readonly`）
- [x] `app.js`：還原 `applyScope` 的 custom 短路
- [x] `app.js`：還原日期欄位的 `change` 監聽，手改起訖即切為 custom
- [x] `app.js`：`restoreFilter` 改為只有 custom 讀回存檔日期，其他範圍一律重算
- [x] `store.rs`：新增 sprint 基準的 key 常數、預設編號與 `default_sprint_anchor_start`
- [x] `store.rs`：`set_setting` 攔空白值，sprint 基準被清空時回填預設
- [x] `db.rs`：`seed` 把 sprint 基準加進 `defaults`，並補回既有 db 內已存成空字串的基準
- [x] `store.rs`：改寫 `resolve_range_falls_back_when_sprint_is_not_configured`（前提已不成立）
- [x] `store.rs`：新增 `clearing_the_sprint_anchor_restores_the_default`
- [x] `db.rs`：新增 `seeds_a_sprint_anchor_on_the_current_week_monday`
- [x] `db.rs`：新增 `seeding_refills_a_sprint_anchor_that_was_cleared`
- [x] 以 cargo test 驗證新增與改寫的測試

## Spec

### PMSpec

- 本 task 無 PM 規格書，需求來源為使用者口述：篩選條件原為今日／本週／本月／本 sprint／上個 sprint／自訂，改為上週、本週、下週、上個 sprint、本 sprint、下個 sprint、上月、本月、下月九個；其後追加「今日也保留」共十個。自訂區間保留。sprint 沒設定基準時要有預設值（當前時間的那個禮拜一），直接存進 db，使其不存在沒有基準值的情境。
- 使用者補充的使用情境：此工具同時用於事前規劃，會提前安排工作與設定工時，因此未來區間（下週、下月、下個 sprint）的「尚缺」是有效資訊，不是無意義的數字。

### UiSpec

#### 搜尋列範圍下拉 — `#rangeScope`

清單十一項，順序固定：

- 今日、上週、本週、下週、上個 sprint、本 sprint、下個 sprint、上月、本月、下月、自訂
- English：Today、Last week、This week、Next week、Last sprint、Current sprint、Next sprint、Last month、This month、Next month、Custom

行為：

- 選十個預設範圍之一 → 後端算出起訖日並回填兩個日期欄位。
- 選「自訂」→ 不動日期欄位，只是讓目前的起訖日脫離預設範圍。
- 手改任一日期欄位 → 下拉自動切成「自訂」並重查。
- 日期欄位可直接編輯（不是唯讀顯示）。

#### 週、月、sprint 的前後推移

- 週：週一至週日，上週為本週起始日往前一天所在的週，下週為本週結束日往後一天所在的週。
- 月：當月一日至月底，上月為當月一日往前一天所在的月，下月為月底往後一天所在的月。
- sprint：依設定的基準編號、基準起始日與每個 sprint 天數推算，上／下個 sprint 為當前編號減一／加一。

#### Sprint 基準預設值

- 基準 sprint 編號預設 `1`。
- 基準起始日預設為當前時間所在那一週的禮拜一（等同「本週」範圍的起始日）。
- 開新資料庫時寫入；既有資料庫缺這兩個 key、或值為空字串時，於開啟時補上。
- 設定頁把基準清空後存檔，寫入端會立即回填預設值。

## Scope

- 搜尋列範圍下拉改為十個預設範圍加自訂，後端 `resolve_range` 同步支援前後推移。
- sprint 基準改為一定有值，涵蓋開新庫、既有庫補值、設定頁清空三條路徑。
- 不含：提醒範圍（`#hoursBadge`）的候選仍是今日／本週／本 sprint 三項，不隨本次新增的範圍擴充。
- 不含：兩個徽章的重複顯示，使用者決定兩個都先保留。
- 不含：Survey 列出的既有缺陷（下拉標籤錯配、fallback 誤標、新增紀錄日期取值等），本次只記錄不修。
- 不含：`entries` 與 schema 改動，`hours_report` 的計算口徑不變。

## Survey 現況調查

### 範圍與查詢的關係

- 查詢條件只看日期，不看 scope：`currentQuery()` 只送 `date_from`／`date_to`（`src/app.js:58`），`build_filter` 也只讀這兩個值，對 scope 一無所知（`src-tauri/src/query.rs:56`、`src-tauri/src/query.rs:62`）。
- scope 的唯一作用是 `applyScope()` 呼叫 `resolve_range` 算出起訖日、回填日期欄位（`src/app.js:683`）。因此「選了今日又手改起訖」一定吃手改的起訖，下拉只是標籤。
- 起訖日為空字串時後端不加該條件（`src-tauri/src/query.rs:57`、`:63`），等同不限日期。
- `resolve_range` 回傳 `from`／`to` 為 `None` 時代表無法解析（未知 scope、或 sprint 無基準）（`src-tauri/src/store.rs:543`）。

### 兩個時數徽章的分工

- `#rangeShortBadge`（左）回答「我正在看的區間」缺多少，一律以 `custom` 帶搜尋列起訖呼叫 `hours_report`（`src/app.js:590`）。
- `#hoursBadge`（右）回答「不管我在看哪裡，今日／本週／當前 sprint」缺多少，對 `REMINDER_SCOPES` 三個範圍各呼叫一次（`src/app.js:588`、`src/app.js:625`）。它是不隨瀏覽區間漂移的固定錨點。
- `#rangeShortBadge` 在關鍵字非空或有狀態過濾時整個隱藏（`src/app.js:594`）；`#hoursBadge` 不受影響。因此搜尋狀態下只有右側徽章還在提醒。
- 系統匣 tooltip 的文字只有 `#hoursBadge` 在餵：`API.updateTray(active.text, ...)`（`src/app.js:651`），`active` 即右側徽章當前選中項；`tooltip` 是系統匣唯一的資訊出口（`src-tauri/src/tray.rs:72`）。視窗可隱藏至系統匣（`src-tauri/src/tray.rs:95`），關窗後 tooltip 是唯一提醒管道。
- 兩者的重複只發生在「下拉範圍與提醒範圍指向同一區間」時（例如兩邊都是本週）。
- `badgeText` 只會收到 `REMINDER_SCOPES` 的三個 scope，所以 `SPRINT_SCOPES` 對 `last_sprint`／`next_sprint` 的分支目前走不到，屬防禦性寫法（`src/app.js:561`）。

### Sprint 基準與預設值

- `sprint_config` 要求基準編號與基準起始日兩者都有值才回 `Some`，任一為空即視為未設定（`src-tauri/src/store.rs:518`）。所以只給起始日預設是不夠的，編號也必須有預設。
- `db.rs` 的 `seed()` 用 `INSERT OR IGNORE`，key 不存在才寫入，所以把預設值加進 `defaults` 陣列即可同時覆蓋開新庫與既有庫缺 key 的情況，不需要另寫 migration（`src-tauri/src/db.rs:106`）。
- 但既有庫可能已有該 key、值卻是空字串（設定頁清空後存檔），`INSERT OR IGNORE` 不會動它，所以 `seed()` 末端另跑 `UPDATE ... WHERE TRIM(value) = ''` 補值（`src-tauri/src/db.rs:147`）。
- 設定頁直接把輸入框的值存進去、不做驗證（`src/settings.js:442`～`444`、`src-tauri/src/commands.rs:68`），所以空值防線放在 `set_setting` 的 `filled_setting`（`src-tauri/src/store.rs:27`），這樣不必等重開 app。
- 預設基準為「本週一、編號 1」，因此早於本週一的日期會算出 sprint 0、-1、-2 等負編號。三個顯示點都不會露出：下拉用「上個 sprint」不帶編號、`hours.sprint {n}` 只會收到當前 sprint（今天必定落在 sprint 1）、`#rangeShortBadge` 不顯示 sprint 字樣。
- `sprint_length_days` 早已有 14 的預設與 fallback（`src-tauri/src/store.rs` 的 `sprint_settings`），本次未改。
- 新增與改寫的測試已以 `cargo test` 執行：全套 75 passed、0 failed，四個新測試全綠。

### 既有缺陷（本次只記錄，未修）

- 下拉標籤與實際查詢可能錯配：`applyScope()` 開頭就寫入 `state.scope`，但 `resolveRange` 失敗或 `range.date_from` 為空時直接 `return`，日期欄位維持前一個範圍的值，而下拉已經顯示新範圍，`rememberFilter()` 還會把這個錯配組合存起來（`src/app.js:683`）。sprint 基準有了預設值之後，`date_from` 為空這條路徑實務上不再發生，但 IPC 失敗那條仍在。
- 提醒徽章在後端 fallback 時會謊報範圍：`hours_report` 解析不到範圍時改用傳入的 `custom_from`／`custom_to`（前端傳的是日期欄位的值），但 `resolved.scope` 原封不動，`badgeText` 照著標成「本 sprint」，數字其實是當前篩選區間的（`src-tauri/src/store.rs:601`）。最小修法是 fallback 成立時把 `resolved.scope` 改寫成 `"custom"`，讓前端顯示「查詢區間」。
- 新增紀錄的工作日期取區間結束日：`work_date: dom.dateTo.value || UI.today()`（`src/app.js:487`）。選「下月」新增會記成下月最後一天，選「上週」記成上週日。原本只有今日／本週／本月時已偏，新增三個未來範圍後誤填機率升高。合理修法是今天落在區間內就用今天，今天晚於區間用 `date_to`，早於區間用 `date_from`。
- 起訖日沒有先後檢查：`date_from` 晚於 `date_to` 會查不到任何資料且無提示，兩者是獨立的 SQL 條件（`src-tauri/src/query.rs:56`～`:66`）。本次改動之前就存在。
- `resolve_range` 未知 scope 的錯誤訊息張冠李戴：未知字串落到 `_ => (None, None)`，前端只看 `!range.date_from` 就 toast「尚未設定 sprint 基準」（`src/app.js:692`）。目前沒有路徑會觸發，custom 在前端就短路了。
- `store.rs` 的週推移算法繞路：`last_week` 是「先算本週範圍、起始日退一天、再算那天的週範圍」，等價於 `today` 減七天；且用 `chrono::Duration::days` 完整路徑，而檔頭第 3 行已 `use chrono::{Duration, ...}`。行為正確，純風格。月份那段的繞法則是必要的，不能用天數加減跨月。

### 本次改動的來源

- `resolve_range` 的週／月／sprint 前後推移、`index.html` 兩個 `readonly`、`app.js` 移除 custom 的各處分支，是另一個 session（`worklog-78`）先做的半成品，且已 `git add` 進暫存區。本次在其上補完，並還原了 custom 相關的全部行為。
- 期間該 session 又執行過 `git add`，使暫存區與工作區版本不一致（暫存區曾留著帶 `readonly` 的 `index.html`）。commit 前需重新 `git add`，以暫存區內容為準的提交才會反映最終狀態。

## Design 設計

1. 下拉清單由 `SCOPES` 陣列的順序決定，`custom` 固定在最後一項（`src/app.js:14`）。
2. 前後推移一律在後端 `resolve_range` 算，前端只負責把回傳的起訖日填進欄位，計算口徑與既有範圍一致。
3. 週與 sprint 用「起始日退一天／結束日進一天」再重算所屬區間；月份必須用這個繞法，不能用固定天數加減。
4. sprint 基準的預設值寫進 db，而不是只在 runtime fallback，這樣設定頁打開就看得到值、使用者能直接改。
5. 空值防線放在 `set_setting`，是所有設定寫入的唯一入口；`db.rs` 的 `seed` 只負責開庫與既有庫補值。
6. `db.rs` 引用 `store` 的 key 常數與預設值函式，避免預設值在兩處各寫一份而漂移。依賴方向為 db → store 單向，store 的非測試程式碼不引用 db，沒有循環。

> 設計決策（已定案）：今日保留，清單共十個預設範圍加自訂。理由：使用者評估雖然少用但仍有用途。

> 設計決策（已定案）：自訂常駐為清單第十一項，不採「手改後臨時插入」或「下拉留空白」。理由：最直白，使用者不必從空白下拉推斷目前狀態。

> 設計決策（已定案）：`restoreFilter` 只有 custom 讀回存檔的起訖日，其他十個範圍一律重算。理由：原本只要存過日期就不重算，導致選「今日」隔天開啟仍顯示前一天的日期，下拉標籤與日期不符。

> 設計決策（已定案）：sprint 基準起始日的預設值取「當前時間所在那一週的禮拜一」，而非「當月第一個禮拜一」。理由：與下拉「本週」的起始日對齊，首次開啟時「本 sprint」必定包含今天，且直接重用 `calendar::week_range`，不需新 helper。

> 設計決策（已定案）：基準 sprint 編號預設 `1`。理由：中性佔位值，讓 `sprint_config` 成立即可，使用者會到設定頁填真實編號。

> 設計決策（已定案）：`#hoursBadge` 與 `#rangeShortBadge` 兩個徽章都保留，不刪除。理由：兩者回答不同問題（固定錨點 vs 當前瀏覽區間）、`#rangeShortBadge` 在搜尋狀態下會隱藏、且系統匣 tooltip 只有 `#hoursBadge` 在餵。

> 設計決策（已定案）：提醒範圍不隨本次新增的範圍擴充，`REMINDER_SCOPES` 維持今日／本週／本 sprint。理由：提醒要的是固定錨點，上月、下週這類範圍不適合當長期提醒目標。

### 資料流

```text
[#rangeScope 選預設範圍]
   │  applyScope(scope) → API.resolveRange(scope)
   ▼
store::resolve_range → calendar::week_range / month_range / SprintConfig::range_of
   │  回傳 date_from / date_to
   ▼
[#dateFrom / #dateTo 回填]
   │
   ├─ 使用者手改任一欄 → state.scope = 'custom'，下拉切為「自訂」
   ▼
currentQuery() → API.searchEntries（只帶 date_from / date_to）
   ▼
query::build_filter → e.work_date >= ? AND e.work_date <= ?
```

```text
[開啟資料庫] db::prepare → seed
   │  INSERT OR IGNORE 寫入基準預設值（缺 key 時）
   │  UPDATE ... WHERE TRIM(value) = '' 補回空字串（既有庫）
   ▼
[設定頁存檔] API.setSetting → store::set_setting → filled_setting
   │  空白值 → 基準編號回填 1、基準起始日回填本週一
   ▼
sprint_config 恆為 Some → resolve_range 的 sprint 分支恆能算出範圍
```

## Wait Decision

以下為使用者指示「以後再說」的待決事項。

- 下拉標籤與實際查詢錯配是否要修（Survey「既有缺陷」第一項）
  - 代價：`applyScope` 需記下前一個 scope，在兩個失敗出口還原 `state.scope` 與 `dom.rangeScope.value`。
  - 建議：修。sprint 基準有預設值後觸發機率大減，但 IPC 失敗仍會留下錯配並被存進 `last_filter`。
- 提醒徽章在後端 fallback 時謊報範圍是否要修（第二項）
  - 代價：改 `hours_report` 在 fallback 成立時把 `resolved.scope` 改寫成 `"custom"`，前端不必改。
  - 建議：修。前端無從得知後端 fallback 了，只能在後端處理。
- 新增紀錄的工作日期取值是否要改（第三項）
  - 代價：`createEntry` 的 `work_date` 改為依今天與區間的相對位置取值。
  - 建議：修。新增三個未來範圍後，這是最容易產生錯誤資料的一處。
- 兩個時數徽章的重複顯示是否要去重
  - 代價：在下拉範圍與提醒範圍指向同一區間時隱藏 `#rangeShortBadge`。
  - 建議：可做。使用者已決定兩個都保留，去重只處理重複、不移除任何功能。
- 起訖日先後檢查要不要加
  - 代價：日期欄 `change` 時比對，或在後端補驗證並回錯誤。
  - 建議：低優先。本次改動之前就存在。
- `REMINDER_SCOPES` 是否放寬到新增的範圍
  - 代價：`app.js` 與 `settings.js` 兩處的 `REMINDER_SCOPES` 要同步，並確認 `badgeText` 對各範圍的文案。
  - 建議：暫不動。已定案提醒維持三個固定錨點，若將來要「上個 sprint」當提醒（月初回頭補登）再議。
