const state = {
  settings: {},
  users: [],
  statuses: [],
  entries: [],
  selectedId: null,
  statusFilter: new Set(),
  scope: 'week',
  draftId: null
};

const SCOPES = ['today', 'week', 'month', 'sprint', 'last_sprint', 'custom'];

const dom = {};

function cache() {
  const ids = [
    'keyword', 'rangeScope', 'dateFrom', 'dateTo', 'statusFilterBtn', 'userFilter',
    'hoursBadge', 'newBtn', 'settingsBtn', 'listCount', 'list', 'detailEmpty', 'detailForm',
    'fTitle', 'fTicket', 'fWorkDate', 'fStatus', 'fHours', 'fUser', 'fContent',
    'saveBtn', 'deleteBtn', 'addLinkBtn', 'linksRelated',
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
    user_id: selectedUserId()
  };
}

function fillScopeSelect() {
  UI.clear(dom.rangeScope);
  SCOPES.forEach((scope) => {
    const option = document.createElement('option');
    option.value = scope;
    option.textContent = I18N.t(`range.${scope}`);
    dom.rangeScope.appendChild(option);
  });
  dom.rangeScope.value = state.scope;
}

function fillUserSelects() {
  const keep = dom.userFilter.value;
  UI.clear(dom.userFilter);
  const any = document.createElement('option');
  any.value = 'any';
  any.textContent = I18N.t('filter.allUsers');
  dom.userFilter.appendChild(any);
  state.users.forEach((user) => {
    const option = document.createElement('option');
    option.value = String(user.id);
    option.textContent = user.name;
    dom.userFilter.appendChild(option);
  });
  const unowned = document.createElement('option');
  unowned.value = 'unowned';
  unowned.textContent = I18N.t('filter.unowned');
  dom.userFilter.appendChild(unowned);
  if (keep) {
    dom.userFilter.value = keep;
  }

  UI.clear(dom.fUser);
  const none = document.createElement('option');
  none.value = '';
  none.textContent = I18N.t('detail.unowned');
  dom.fUser.appendChild(none);
  state.users.forEach((user) => {
    const option = document.createElement('option');
    option.value = String(user.id);
    option.textContent = user.name;
    dom.fUser.appendChild(option);
  });
}

function fillStatusSelect(select) {
  UI.clear(select);
  state.statuses.forEach((status) => {
    const option = document.createElement('option');
    option.value = String(status.id);
    option.textContent = status.name;
    select.appendChild(option);
  });
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

function renderList() {
  UI.clear(dom.list);
  dom.listCount.textContent = I18N.t('list.count', { n: state.entries.length });

  if (state.entries.length === 0) {
    const empty = UI.el('div', 'empty');
    empty.appendChild(UI.el('span', null, I18N.t('empty.list')));
    dom.list.appendChild(empty);
    return;
  }

  state.entries.forEach((entry) => {
    const row = UI.el('div', 'row');
    row.dataset.id = String(entry.id);
    if (entry.id === state.selectedId) {
      row.classList.add('selected');
    }

    row.appendChild(UI.el('div', 'row-ticket', entry.ticket));
    row.appendChild(UI.el('div', 'row-title', entry.title));
    row.appendChild(UI.el('div', 'row-hours', `${UI.hours(entry.hours)}h`));

    const meta = UI.el('div', 'row-meta');
    meta.appendChild(UI.dot(entry.status_color));

    const statusSelect = document.createElement('select');
    statusSelect.className = 'row-status';
    state.statuses.forEach((status) => {
      const option = document.createElement('option');
      option.value = String(status.id);
      option.textContent = status.name;
      statusSelect.appendChild(option);
    });
    statusSelect.value = String(entry.status_id);
    statusSelect.addEventListener('click', (event) => event.stopPropagation());
    statusSelect.addEventListener('change', async (event) => {
      event.stopPropagation();
      await UI.guard(() => API.setEntryStatus(entry.id, Number(statusSelect.value)));
      await refresh();
    });
    meta.appendChild(statusSelect);

    meta.appendChild(UI.el('span', 'num', entry.work_date));
    if (entry.user_name) {
      meta.appendChild(UI.el('span', null, entry.user_name));
    }
    row.appendChild(meta);

    row.addEventListener('click', () => select(entry.id));
    dom.list.appendChild(row);
  });
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

  const entry = detail.entry;
  dom.fTitle.value = entry.title;
  dom.fTicket.value = entry.ticket;
  dom.fWorkDate.value = entry.work_date;
  dom.fHours.value = entry.hours;
  dom.fContent.value = entry.content;
  fillStatusSelect(dom.fStatus);
  dom.fStatus.value = String(entry.status_id);
  dom.fUser.value = entry.user_id === null ? '' : String(entry.user_id);

  renderLinks(detail.related);
  showForm(true);
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

function startDraft() {
  state.selectedId = null;
  state.draftId = null;
  dom.fTitle.value = '';
  dom.fTicket.value = '';
  dom.fWorkDate.value = dom.dateTo.value || UI.today();
  dom.fHours.value = 0;
  dom.fContent.value = '';
  fillStatusSelect(dom.fStatus);
  const fallback = state.settings.default_status_id;
  if (fallback) {
    dom.fStatus.value = fallback;
  }
  dom.fUser.value = draftUserValue();
  renderLinks([]);
  showForm(true);
  renderList();
  dom.fTitle.focus();
}

async function select(id) {
  state.selectedId = id;
  renderList();
  await loadDetail(id);
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

async function refreshBadge() {
  const scope = state.settings.reminder_range || 'week';
  const report = await UI.guard(() =>
    API.hoursReport(selectedUserId(), scope, dom.dateFrom.value, dom.dateTo.value)
  );
  if (!report) {
    return;
  }
  const { text, tone } = badgeText(report);
  dom.hoursBadge.textContent = text;
  dom.hoursBadge.className = `badge ${tone}`.trim();
  await UI.guard(() => API.updateTray(text, I18N.t('tray.show'), I18N.t('tray.quit')));
}

async function refresh(selectId) {
  const entries = await UI.guard(() => API.searchEntries(currentQuery()));
  if (!entries) {
    return;
  }
  state.entries = entries;

  if (selectId !== undefined) {
    state.selectedId = selectId;
  }
  if (state.selectedId !== null && !entries.some((entry) => entry.id === state.selectedId)) {
    if (selectId === undefined) {
      state.selectedId = null;
      showForm(false);
    }
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

async function save() {
  const input = {
    id: state.selectedId,
    user_id: dom.fUser.value === '' ? null : Number(dom.fUser.value),
    title: dom.fTitle.value,
    content: dom.fContent.value,
    ticket: dom.fTicket.value,
    status_id: dom.fStatus.value === '' ? null : Number(dom.fStatus.value),
    work_date: dom.fWorkDate.value,
    hours: Number(dom.fHours.value) || 0
  };
  const id = await UI.guard(() => API.saveEntry(input));
  if (id === undefined) {
    return;
  }
  UI.toast(I18N.t('toast.saved'));
  await refresh(id);
  scrollToSelected();
}

async function remove() {
  if (state.selectedId === null) {
    return;
  }
  if (!window.confirm(I18N.t('confirm.deleteEntry'))) {
    return;
  }
  await UI.guard(() => API.deleteEntry(state.selectedId));
  state.selectedId = null;
  showForm(false);
  UI.toast(I18N.t('toast.deleted'));
  await refresh();
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
  const items = await UI.guard(() =>
    API.linkCandidates(state.selectedId, dom.linkSearch.value)
  );
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
  const clamped = Math.min(Math.max(width, 200), 420);
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
    user: dom.userFilter.value
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
  if (saved.user && [...dom.userFilter.options].some((option) => option.value === saved.user)) {
    dom.userFilter.value = saved.user;
  }
  renderStatusFilterButton();
  renderStatusChecks();
  return Boolean(saved.dateFrom || saved.dateTo || state.scope === 'custom');
}

function applyLanguageAndTheme() {
  I18N.set(state.settings.language || 'zh-TW');
  UI.applyTheme(state.settings.theme || 'dark');
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

  dom.newBtn.addEventListener('click', () => startDraft());
  dom.settingsBtn.addEventListener('click', () =>
    UI.guard(() => API.openSettingsWindow(state.settings.theme || 'dark'))
  );

  dom.saveBtn.addEventListener('click', () => save());
  dom.deleteBtn.addEventListener('click', () => remove());

  dom.addLinkBtn.addEventListener('click', () => openLinkPicker());
  dom.linkDialogClose.addEventListener('click', () => dom.linkDialog.close());
  dom.linkSearch.addEventListener('input', debounce(() => renderCandidates(), 200));

  bindSplitter();

  window.addEventListener('focus', () => reloadSettings());

  document.addEventListener('keydown', (event) => {
    if (event.ctrlKey && event.key === 's') {
      event.preventDefault();
      if (!dom.detailForm.hidden) {
        save();
      }
    }
    if (event.ctrlKey && event.key === 'n') {
      event.preventDefault();
      startDraft();
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

  const datesRestored = restoreFilter();
  if (!datesRestored) {
    await applyScope(state.scope);
  }
  await refresh();
}

window.addEventListener('DOMContentLoaded', init);
