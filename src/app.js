const state = {
  settings: {},
  users: [],
  statuses: [],
  entries: [],
  selectedId: null,
  editingId: null,
  statusFilter: new Set(),
  scope: 'week',
  sortField: 'work_date',
  sortDir: 'desc'
};

const SCOPES = ['today', 'week', 'month', 'sprint', 'last_sprint', 'custom'];

const dom = {};

function cache() {
  const ids = [
    'keyword', 'rangeScope', 'dateFrom', 'dateTo', 'statusFilterBtn', 'userFilter',
    'hoursBadge', 'settingsBtn', 'listCount', 'listHead', 'list', 'detailEmpty', 'detailForm',
    'fContent', 'addLinkBtn', 'linksRelated',
    'statusDialog', 'statusChecks', 'statusDialogClose',
    'linkDialog', 'linkSearch', 'linkCandidates', 'linkDialogClose',
    'splitter'
  ];
  ids.forEach((id) => {
    dom[id] = document.getElementById(id);
  });
}

function debounce(fn, wait) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), wait);
  };
}

function selectedUserId() {
  const value = dom.userFilter.value;
  return value === 'any' || value === 'unowned' ? null : Number(value);
}

function currentQuery() {
  const value = dom.userFilter.value;
  return {
    keyword: dom.keyword.value,
    date_from: dom.dateFrom.value,
    date_to: dom.dateTo.value,
    status_ids: [...state.statusFilter],
    user_filter: value === 'any' ? 'any' : value === 'unowned' ? 'unowned' : 'one',
    user_id: selectedUserId(),
    sort_field: state.sortField,
    sort_dir: state.sortDir
  };
}

function option(value, label) {
  const node = document.createElement('option');
  node.value = value;
  node.textContent = label;
  return node;
}

function fillScopeSelect() {
  UI.clear(dom.rangeScope);
  SCOPES.forEach((scope) => dom.rangeScope.appendChild(option(scope, I18N.t(`range.${scope}`))));
  dom.rangeScope.value = state.scope;
}

function fillUserSelects() {
  const keep = dom.userFilter.value;
  UI.clear(dom.userFilter);
  dom.userFilter.appendChild(option('any', I18N.t('filter.allUsers')));
  state.users.forEach((user) => dom.userFilter.appendChild(option(String(user.id), user.name)));
  dom.userFilter.appendChild(option('unowned', I18N.t('filter.unowned')));
  if (keep) {
    dom.userFilter.value = keep;
  }
}

function statusSelect(className, statusId) {
  const select = document.createElement('select');
  select.className = className;
  state.statuses.forEach((status) => select.appendChild(option(String(status.id), status.name)));
  select.value = String(statusId);
  return select;
}

function userSelect(className, userId) {
  const select = document.createElement('select');
  select.className = className;
  select.appendChild(option('', I18N.t('detail.unowned')));
  state.users.forEach((user) => select.appendChild(option(String(user.id), user.name)));
  select.value = userId === null || userId === undefined ? '' : String(userId);
  return select;
}

function renderStatusFilterButton() {
  const count = state.statusFilter.size;
  dom.statusFilterBtn.textContent =
    count === 0 ? I18N.t('filter.allStatus') : `${I18N.t('filter.status')} (${count})`;
}

function renderStatusChecks() {
  UI.clear(dom.statusChecks);
  state.statuses.forEach((status) => {
    const label = UI.el('label', 'check');
    const box = document.createElement('input');
    box.type = 'checkbox';
    box.checked = state.statusFilter.has(status.id);
    box.addEventListener('change', () => {
      if (box.checked) {
        state.statusFilter.add(status.id);
      } else {
        state.statusFilter.delete(status.id);
      }
      renderStatusFilterButton();
      refresh();
    });
    label.appendChild(box);
    label.appendChild(UI.dot(status.color));
    label.appendChild(UI.el('span', null, status.name));
    dom.statusChecks.appendChild(label);
  });
}

function field(className, value, type) {
  const node = document.createElement('input');
  node.className = className;
  if (type) {
    node.type = type;
  }
  node.value = value ?? '';
  return node;
}

function buildEditRow(entry) {
  const row = UI.el('div', 'row selected editing');
  row.dataset.id = String(entry.id);

  const ticket = field('c-ticket edit-cell', entry.ticket);
  ticket.placeholder = I18N.t('detail.ticket');
  const title = field('c-title edit-cell', entry.title);
  title.placeholder = I18N.t('detail.title');
  const hours = field('c-hours edit-cell', entry.hours, 'number');
  hours.step = '0.5';
  hours.min = '0';
  const status = statusSelect('c-status edit-cell', entry.status_id);
  const date = field('c-date edit-cell', entry.work_date, 'date');
  const user = userSelect('c-user edit-cell', entry.user_id);

  const remove = UI.el('button', 'c-remove', '✕');
  remove.title = I18N.t('btn.delete');
  remove.addEventListener('click', (event) => {
    event.stopPropagation();
    removeEntry(entry.id);
  });

  row.appendChild(ticket);
  row.appendChild(title);
  row.appendChild(hours);
  row.appendChild(UI.dot(entry.status_color));
  row.appendChild(status);
  row.appendChild(date);
  row.appendChild(user);
  row.appendChild(remove);

  [ticket, title, hours, status, date, user].forEach((node) => {
    node.addEventListener('change', () => saveSelected());
  });

  row.addEventListener('click', (event) => event.stopPropagation());
  return row;
}

function buildDisplayRow(entry) {
  const row = UI.el('div', 'row');
  row.dataset.id = String(entry.id);
  if (entry.id === state.selectedId) {
    row.classList.add('selected');
  }

  row.appendChild(UI.el('span', 'c-ticket', entry.ticket));
  row.appendChild(UI.el('span', 'c-title', entry.title));
  row.appendChild(UI.el('span', 'c-hours', UI.hours(entry.hours)));
  row.appendChild(UI.dot(entry.status_color));

  const status = statusSelect('c-status', entry.status_id);
  status.addEventListener('click', (event) => event.stopPropagation());
  status.addEventListener('change', async (event) => {
    event.stopPropagation();
    await UI.guard(() => API.setEntryStatus(entry.id, Number(status.value)));
    await refresh();
  });
  row.appendChild(status);

  row.appendChild(UI.el('span', 'c-date num', entry.work_date));
  row.appendChild(UI.el('span', 'c-user', entry.user_name ?? ''));
  row.appendChild(UI.el('span', 'c-remove-spacer'));

  row.addEventListener('click', (event) => {
    event.stopPropagation();
    select(entry.id);
  });
  return row;
}

const DEFAULT_COLUMNS = { ticket: 56, hours: 40, status: 68, date: 94, user: 68 };
const COLUMN_LIMITS = { min: 32, max: 220 };

function applyColumns(columns) {
  const merged = { ...DEFAULT_COLUMNS, ...(columns || {}) };
  Object.entries(merged).forEach(([name, width]) => {
    const value = Math.min(Math.max(Number(width) || DEFAULT_COLUMNS[name], COLUMN_LIMITS.min), COLUMN_LIMITS.max);
    document.documentElement.style.setProperty(`--w-${name}`, `${value}px`);
  });
}

function readColumn(name) {
  const raw = getComputedStyle(document.documentElement).getPropertyValue(`--w-${name}`);
  return parseFloat(raw) || DEFAULT_COLUMNS[name];
}

async function saveColumns() {
  const layout = parseJson(state.settings.window_layout, {});
  layout.columns = Object.fromEntries(
    Object.keys(DEFAULT_COLUMNS).map((name) => [name, readColumn(name)])
  );
  state.settings.window_layout = JSON.stringify(layout);
  await UI.guard(() => API.setSetting('window_layout', state.settings.window_layout));
}

function columnHandle(name) {
  const handle = UI.el('span', 'col-resize');
  handle.title = I18N.t('sort.resize');
  handle.addEventListener('click', (event) => event.stopPropagation());
  handle.addEventListener('mousedown', (event) => {
    event.preventDefault();
    event.stopPropagation();
    handle.classList.add('dragging');

    const startX = event.clientX;
    const startWidth = readColumn(name);

    const move = (moveEvent) => {
      const next = startWidth + (moveEvent.clientX - startX);
      applyColumns({ [name]: next });
    };
    const up = async () => {
      window.removeEventListener('mousemove', move);
      window.removeEventListener('mouseup', up);
      handle.classList.remove('dragging');
      await saveColumns();
    };
    window.addEventListener('mousemove', move);
    window.addEventListener('mouseup', up);
  });
  return handle;
}

const SORT_COLUMNS = [
  { field: 'ticket', label: 'detail.ticket', cell: 'c-ticket', first: 'desc', resize: 'ticket' },
  { field: 'title', label: 'detail.title', cell: 'c-title', first: 'asc' },
  { field: 'hours', label: 'detail.hours', cell: 'c-hours', first: 'desc', resize: 'hours' },
  { field: 'status', label: 'detail.status', cell: 'c-status', first: 'asc', resize: 'status' },
  { field: 'work_date', label: 'detail.workDate', cell: 'c-date', first: 'desc', resize: 'date' },
  { field: 'user', label: 'detail.user', cell: 'c-user', first: 'asc', resize: 'user' }
];

function sortButton(column) {
  const button = UI.el('button', `sort-cell ${column.cell}`);
  button.appendChild(UI.el('span', 'sort-label', I18N.t(column.label)));

  const active = state.sortField === column.field;
  button.appendChild(
    UI.el('span', 'sort-arrow', active ? (state.sortDir === 'asc' ? '▲' : '▼') : '')
  );
  if (active) {
    button.classList.add('active');
  }
  button.title = I18N.t('sort.hint');

  button.addEventListener('click', async () => {
    if (state.sortField === column.field) {
      state.sortDir = state.sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      state.sortField = column.field;
      state.sortDir = column.first;
    }
    await refresh();
  });

  if (column.resize) {
    button.appendChild(columnHandle(column.resize));
  }
  return button;
}

function renderHead() {
  UI.clear(dom.listHead);
  const [ticket, title, hours, status, workDate, user] = SORT_COLUMNS;

  dom.listHead.appendChild(sortButton(ticket));
  dom.listHead.appendChild(sortButton(title));
  dom.listHead.appendChild(sortButton(hours));
  dom.listHead.appendChild(UI.el('span', 'dot-spacer'));
  dom.listHead.appendChild(sortButton(status));
  dom.listHead.appendChild(sortButton(workDate));
  dom.listHead.appendChild(sortButton(user));
  dom.listHead.appendChild(UI.el('span', 'c-remove-spacer'));
}

function renderList() {
  renderHead();
  UI.clear(dom.list);
  dom.listCount.textContent = I18N.t('list.count', { n: state.entries.length });

  if (state.entries.length === 0) {
    UI.clear(dom.list);
    const empty = UI.el('div', 'empty');
    empty.appendChild(UI.el('span', null, I18N.t('empty.list')));
    dom.list.appendChild(empty);
    dom.list.appendChild(buildAddButton());
    return;
  }

  state.entries.forEach((entry) => {
    dom.list.appendChild(
      entry.id === state.editingId ? buildEditRow(entry) : buildDisplayRow(entry)
    );
  });

  dom.list.appendChild(buildAddButton());
}

function buildAddButton() {
  const wrap = UI.el('div', 'add-row');
  const button = UI.el('button', 'add-box', '＋');
  button.title = I18N.t('quick.add');
  button.addEventListener('click', (event) => {
    event.stopPropagation();
    createEntry('');
    button.focus();
  });
  wrap.appendChild(button);
  return wrap;
}

function renderLinks(items) {
  UI.clear(dom.linksRelated);
  if (items.length === 0) {
    dom.linksRelated.appendChild(UI.el('div', 'hint', I18N.t('links.none')));
    return;
  }
  items.forEach((item) => {
    const row = UI.el('div', 'link-row');
    row.appendChild(UI.dot(item.status_color));
    if (item.ticket) {
      row.appendChild(UI.el('span', 'link-ticket', item.ticket));
    }
    row.appendChild(UI.el('span', 'link-title', item.title));
    row.appendChild(UI.el('span', 'link-meta', item.work_date));
    row.addEventListener('click', () => navigateTo(item.id));

    const remove = UI.el('button', 'btn tiny ghost', '✕');
    remove.title = I18N.t('links.remove');
    remove.addEventListener('click', async (event) => {
      event.stopPropagation();
      await UI.guard(() => API.unlinkEntry(item.id));
      await loadDetail(state.selectedId);
    });
    row.appendChild(remove);
    dom.linksRelated.appendChild(row);
  });
}

function showForm(visible) {
  dom.detailForm.hidden = !visible;
  dom.detailEmpty.hidden = visible;
}

async function loadDetail(id) {
  if (id === null) {
    showForm(false);
    return;
  }
  const detail = await UI.guard(() => API.entryDetail(id));
  if (!detail) {
    showForm(false);
    return;
  }
  dom.fContent.value = detail.entry.content;
  renderLinks(detail.related);
  showForm(true);
}

function collectInput() {
  const row = dom.list.querySelector('.row.editing');
  if (!row || state.editingId === null) {
    return null;
  }
  const read = (selector) => {
    const node = row.querySelector(selector);
    return node ? node.value : '';
  };
  return {
    id: state.editingId,
    user_id: read('.c-user') === '' ? null : Number(read('.c-user')),
    title: read('.c-title'),
    content: dom.fContent.value,
    ticket: read('.c-ticket'),
    status_id: Number(read('.c-status')),
    work_date: read('.c-date'),
    hours: Number(read('.c-hours')) || 0
  };
}

async function saveSelected() {
  const payload = collectInput();
  if (!payload) {
    return;
  }
  if (!payload.work_date) {
    payload.work_date = UI.today();
  }
  const id = await UI.guard(() => API.saveEntry(payload));
  if (id === undefined) {
    return;
  }

  const entry = state.entries.find((item) => item.id === id);
  if (entry) {
    const status = state.statuses.find((item) => item.id === payload.status_id);
    const user = state.users.find((item) => item.id === payload.user_id);
    Object.assign(entry, {
      title: payload.title,
      ticket: payload.ticket,
      work_date: payload.work_date,
      hours: payload.hours,
      status_id: payload.status_id,
      status_name: status ? status.name : entry.status_name,
      status_color: status ? status.color : entry.status_color,
      user_id: payload.user_id,
      user_name: user ? user.name : null
    });
  }

  await refreshBadge();
}

function draftUserValue() {
  const filter = dom.userFilter.value;
  if (filter === 'unowned') {
    return '';
  }
  if (filter !== 'any') {
    return filter;
  }
  return state.users.length > 0 ? String(state.users[0].id) : '';
}

async function createEntry(title, options = {}) {
  const owner = draftUserValue();
  const payload = {
    id: null,
    user_id: owner === '' ? null : Number(owner),
    title: title ? title.trim() : '',
    content: '',
    ticket: '',
    status_id: state.settings.default_status_id ? Number(state.settings.default_status_id) : null,
    work_date: dom.dateTo.value || UI.today(),
    hours: 0
  };
  const id = await UI.guard(() => API.saveEntry(payload));
  if (id === undefined) {
    return;
  }
  await refresh(id);
  scrollToSelected();

  if (options.focusTitle) {
    const node = dom.list.querySelector('.row.editing .c-title');
    if (node) {
      node.focus();
      node.select();
    }
  }
}

async function removeEntry(id) {
  if (!window.confirm(I18N.t('confirm.deleteEntry'))) {
    return;
  }
  await UI.guard(() => API.deleteEntry(id));
  state.selectedId = null;
  state.editingId = null;
  showForm(false);
  UI.toast(I18N.t('toast.deleted'));
  await refresh();
}

async function select(id) {
  const leaving = state.editingId !== null && state.editingId !== id;
  state.selectedId = id;
  state.editingId = id;
  if (leaving) {
    await refresh(id);
  } else {
    renderList();
  }
  await loadDetail(id);
}

function stopEditing() {
  if (state.editingId === null) {
    return;
  }
  state.editingId = null;
  refresh();
}

async function navigateTo(id) {
  const visible = state.entries.some((entry) => entry.id === id);
  if (!visible) {
    dom.keyword.value = '';
    state.statusFilter.clear();
    renderStatusFilterButton();
    renderStatusChecks();
    await refresh(id);
    UI.toast(I18N.t('toast.filterRelaxed'));
    scrollToSelected();
    return;
  }
  await select(id);
  scrollToSelected();
}

function scrollToSelected() {
  const row = dom.list.querySelector('.row.selected');
  if (row) {
    row.scrollIntoView({ block: 'nearest' });
  }
}

function badgeText(report) {
  if (!report.has_user) {
    return { text: I18N.t('hours.noUser'), tone: '' };
  }

  let scope;
  if (report.scope === 'sprint' || report.scope === 'last_sprint') {
    scope =
      report.sprint_number === null
        ? I18N.t(`range.${report.scope}`)
        : I18N.t('hours.sprint', { n: report.sprint_number });
  } else if (SCOPES.includes(report.scope) && report.scope !== 'custom') {
    scope = I18N.t(`range.${report.scope}`);
  } else {
    scope = I18N.t('hours.custom');
  }

  const diff = Math.round(report.diff * 100) / 100;
  if (diff < 0) {
    return { text: I18N.t('hours.short', { scope, n: UI.hours(-diff) }), tone: 'short' };
  }
  if (diff === 0) {
    return { text: I18N.t('hours.exact', { scope }), tone: 'exact' };
  }
  return { text: I18N.t('hours.over', { scope, n: UI.hours(diff) }), tone: 'over' };
}

const REMINDER_SCOPES = ['today', 'week', 'sprint'];

async function refreshBadge() {
  const current = REMINDER_SCOPES.includes(state.settings.reminder_range)
    ? state.settings.reminder_range
    : 'week';

  const results = [];
  for (const scope of REMINDER_SCOPES) {
    const report = await UI.guard(() =>
      API.hoursReport(selectedUserId(), scope, dom.dateFrom.value, dom.dateTo.value)
    );
    if (report) {
      results.push({ scope, ...badgeText(report) });
    }
  }
  if (results.length === 0) {
    return;
  }

  UI.clear(dom.hoursBadge);
  results.forEach((item) => dom.hoursBadge.appendChild(option(item.scope, item.text)));
  dom.hoursBadge.value = current;

  const active = results.find((item) => item.scope === current) || results[0];
  dom.hoursBadge.className = `badge ${active.tone}`.trim();
  await UI.guard(() => API.updateTray(active.text, I18N.t('tray.show'), I18N.t('tray.quit')));
}

async function refresh(selectId) {
  const entries = await UI.guard(() => API.searchEntries(currentQuery()));
  if (!entries) {
    return;
  }
  state.entries = entries;

  if (selectId !== undefined) {
    state.selectedId = selectId;
    state.editingId = selectId;
  }
  if (
    selectId === undefined &&
    state.selectedId !== null &&
    !entries.some((entry) => entry.id === state.selectedId)
  ) {
    state.selectedId = null;
    state.editingId = null;
    showForm(false);
  }

  renderList();
  if (state.selectedId !== null) {
    await loadDetail(state.selectedId);
  }
  await refreshBadge();
  rememberFilter();
}

async function applyScope(scope) {
  state.scope = scope;
  if (scope === 'custom') {
    return;
  }
  const range = await UI.guard(() => API.resolveRange(scope));
  if (!range) {
    return;
  }
  if (!range.date_from) {
    UI.toast(I18N.t('sprint.notConfigured'), true);
    return;
  }
  dom.dateFrom.value = range.date_from;
  dom.dateTo.value = range.date_to;
}

async function openLinkPicker() {
  if (state.selectedId === null) {
    return;
  }
  await renderCandidates();
  dom.linkDialog.showModal();
  dom.linkSearch.focus();
}

async function renderCandidates() {
  const items = await UI.guard(() => API.linkCandidates(state.selectedId, dom.linkSearch.value));
  UI.clear(dom.linkCandidates);
  if (!items || items.length === 0) {
    dom.linkCandidates.appendChild(UI.el('div', 'hint', I18N.t('links.noCandidate')));
    return;
  }
  items.forEach((item) => {
    const row = UI.el('div', 'link-row');
    row.appendChild(UI.dot(item.status_color));
    if (item.ticket) {
      row.appendChild(UI.el('span', 'link-ticket', item.ticket));
    }
    row.appendChild(UI.el('span', 'link-title', item.title));
    row.appendChild(UI.el('span', 'link-meta', item.work_date));
    row.addEventListener('click', async () => {
      await UI.guard(() => API.linkEntries(state.selectedId, item.id));
      dom.linkDialog.close();
      await loadDetail(state.selectedId);
    });
    dom.linkCandidates.appendChild(row);
  });
}

function parseJson(raw, fallback) {
  if (!raw) {
    return fallback;
  }
  try {
    return JSON.parse(raw);
  } catch {
    return fallback;
  }
}

function applyListWidth(width) {
  const clamped = Math.min(Math.max(width, 420), 700);
  document.documentElement.style.setProperty('--list-w', `${clamped}px`);
  return clamped;
}

function bindSplitter() {
  let dragging = false;

  dom.splitter.addEventListener('mousedown', (event) => {
    dragging = true;
    dom.splitter.classList.add('dragging');
    event.preventDefault();
  });

  window.addEventListener('mousemove', (event) => {
    if (dragging) {
      applyListWidth(event.clientX);
    }
  });

  window.addEventListener('mouseup', async () => {
    if (!dragging) {
      return;
    }
    dragging = false;
    dom.splitter.classList.remove('dragging');
    const width = parseFloat(
      getComputedStyle(document.documentElement).getPropertyValue('--list-w')
    );
    const layout = parseJson(state.settings.window_layout, {});
    layout.listWidth = width;
    state.settings.window_layout = JSON.stringify(layout);
    await UI.guard(() => API.setSetting('window_layout', state.settings.window_layout));
  });
}

function rememberFilter() {
  const payload = JSON.stringify({
    keyword: dom.keyword.value,
    scope: state.scope,
    dateFrom: dom.dateFrom.value,
    dateTo: dom.dateTo.value,
    statusIds: [...state.statusFilter],
    user: dom.userFilter.value,
    sortField: state.sortField,
    sortDir: state.sortDir
  });
  state.settings.last_filter = payload;
  UI.guard(() => API.setSetting('last_filter', payload));
}

function restoreFilter() {
  const saved = parseJson(state.settings.last_filter, null);
  if (!saved) {
    return false;
  }
  dom.keyword.value = saved.keyword ?? '';
  dom.dateFrom.value = saved.dateFrom ?? '';
  dom.dateTo.value = saved.dateTo ?? '';
  state.scope = SCOPES.includes(saved.scope) ? saved.scope : 'week';
  dom.rangeScope.value = state.scope;
  state.statusFilter = new Set(
    (saved.statusIds ?? []).filter((id) => state.statuses.some((status) => status.id === id))
  );
  if (saved.user && [...dom.userFilter.options].some((item) => item.value === saved.user)) {
    dom.userFilter.value = saved.user;
  }
  if (SORT_COLUMNS.some((column) => column.field === saved.sortField)) {
    state.sortField = saved.sortField;
    state.sortDir = saved.sortDir === 'asc' ? 'asc' : 'desc';
  }
  renderStatusFilterButton();
  renderStatusChecks();
  return Boolean(saved.dateFrom || saved.dateTo || state.scope === 'custom');
}

function applyLanguageAndTheme() {
  I18N.set(state.settings.language || 'zh-TW');
  UI.applyTheme(state.settings.theme || 'dark');
  UI.applyDensity(state.settings.density);
  I18N.apply();
  fillScopeSelect();
  fillUserSelects();
  renderStatusFilterButton();
  renderStatusChecks();
}

function bind() {
  dom.keyword.addEventListener('input', debounce(() => refresh(), 200));

  dom.rangeScope.addEventListener('change', async () => {
    await applyScope(dom.rangeScope.value);
    await refresh();
  });

  [dom.dateFrom, dom.dateTo].forEach((node) => {
    node.addEventListener('change', async () => {
      state.scope = 'custom';
      dom.rangeScope.value = 'custom';
      await refresh();
    });
  });

  dom.statusFilterBtn.addEventListener('click', () => dom.statusDialog.showModal());
  dom.statusDialogClose.addEventListener('click', () => dom.statusDialog.close());

  dom.userFilter.addEventListener('change', () => refresh());

  dom.hoursBadge.addEventListener('change', async () => {
    state.settings.reminder_range = dom.hoursBadge.value;
    await UI.guard(() => API.setSetting('reminder_range', dom.hoursBadge.value));
    await refreshBadge();
  });

  dom.settingsBtn.addEventListener('click', () =>
    UI.guard(() => API.openSettingsWindow(state.settings.theme || 'dark'))
  );

  dom.fContent.addEventListener('change', () => saveSelected());

  dom.addLinkBtn.addEventListener('click', () => openLinkPicker());
  dom.linkDialogClose.addEventListener('click', () => dom.linkDialog.close());
  dom.linkSearch.addEventListener('input', debounce(() => renderCandidates(), 200));

  bindSplitter();

  document.addEventListener('click', (event) => {
    if (event.target.closest('.detail-pane, dialog, .row')) {
      return;
    }
    stopEditing();
  });

  window.addEventListener('focus', () => reloadSettings());

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey && event.key === 'n') {
      event.preventDefault();
      createEntry('', { focusTitle: true });
    }
  });
}

async function reloadSettings() {
  const bootstrap = await UI.guard(() => API.bootstrap());
  if (!bootstrap) {
    return;
  }
  state.settings = bootstrap.settings;
  state.users = bootstrap.users;
  state.statuses = bootstrap.statuses;
  state.statusFilter = new Set(
    [...state.statusFilter].filter((id) => state.statuses.some((status) => status.id === id))
  );
  applyLanguageAndTheme();
  await refresh();
}

async function init() {
  cache();
  bind();

  const bootstrap = await UI.guard(() => API.bootstrap());
  if (!bootstrap) {
    I18N.apply();
    return;
  }
  state.settings = bootstrap.settings;
  state.users = bootstrap.users;
  state.statuses = bootstrap.statuses;

  applyLanguageAndTheme();

  const layout = parseJson(state.settings.window_layout, {});
  if (layout.listWidth) {
    applyListWidth(layout.listWidth);
  }
  applyColumns(layout.columns);

  const datesRestored = restoreFilter();
  if (!datesRestored) {
    await applyScope(state.scope);
  }
  await refresh();
}

window.addEventListener('DOMContentLoaded', init);
