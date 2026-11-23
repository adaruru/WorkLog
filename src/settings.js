const state = {
  settings: {},
  users: [],
  statuses: [],
  pendingDelete: null
};

const THEMES = ['dark', 'light', 'blue'];
const REMINDER_SCOPES = ['today', 'week', 'sprint'];
const DENSITIES = ['compact', 'cozy', 'comfortable'];

const dom = {};

function cache() {
  const ids = [
    'language', 'theme', 'density', 'reminderRange',
    'userTable', 'newUserName', 'newUserHours', 'addUser',
    'defaultStatus', 'statusTable', 'newStatusName', 'newStatusColor', 'addStatus',
    'holidayTable', 'newHolidayDate', 'newHolidayName', 'newHolidayWorkday', 'addHoliday',
    'importYear', 'importTaiwan',
    'leaveTable', 'newLeaveUser', 'newLeaveDate', 'newLeaveHours', 'addLeave',
    'sprintNumber', 'sprintStart', 'sprintLength', 'saveSprint',
    'dbPath', 'changeDb', 'revealDb',
    'statusDeleteDialog', 'replacementStatus', 'statusDeleteCancel', 'statusDeleteConfirm'
  ];
  ids.forEach((id) => {
    dom[id] = document.getElementById(id);
  });
}

function row(columns) {
  const node = UI.el('div', 'table-row');
  node.style.gridTemplateColumns = columns;
  return node;
}

function fillOptions(select, values, labeller, current) {
  UI.clear(select);
  values.forEach((value) => {
    const option = document.createElement('option');
    option.value = value;
    option.textContent = labeller(value);
    select.appendChild(option);
  });
  if (current !== undefined) {
    select.value = current;
  }
}

function renderGeneral() {
  dom.language.value = state.settings.language || 'zh-TW';
  fillOptions(dom.theme, THEMES, (value) => I18N.t(`theme.${value}`), state.settings.theme || 'dark');
  fillOptions(
    dom.density,
    DENSITIES,
    (value) => I18N.t(`density.${value}`),
    state.settings.density || 'cozy'
  );
  fillOptions(
    dom.reminderRange,
    REMINDER_SCOPES,
    (value) => I18N.t(`range.${value}`),
    state.settings.reminder_range || 'week'
  );
}

function renderUsers() {
  UI.clear(dom.userTable);
  const head = row('1fr 120px 28px');
  head.classList.add('head');
  head.appendChild(UI.el('span', null, I18N.t('users.name')));
  head.appendChild(UI.el('span', null, I18N.t('users.dailyHours')));
  head.appendChild(UI.el('span'));
  dom.userTable.appendChild(head);

  if (state.users.length === 0) {
    const empty = row('1fr');
    empty.appendChild(UI.el('span', 'hint', I18N.t('common.empty')));
    dom.userTable.appendChild(empty);
  }

  state.users.forEach((user) => {
    const line = row('1fr 120px 28px');

    const name = document.createElement('input');
    name.className = 'input';
    name.value = user.name;
    name.addEventListener('change', async () => {
      const before = userStateOf(user);
      const after = { ...before, name: name.value };
      await UI.guard(() =>
        API.updateUser(user.id, after.name, after.daily_required_hours, after.is_active)
      );
      if (before.name !== after.name) {
        pushHistory([{ kind: 'user', op: 'update', id: user.id, before, after }]);
      }
      await reload();
    });

    const hours = document.createElement('input');
    hours.className = 'input num';
    hours.type = 'number';
    hours.step = '0.5';
    hours.min = '0';
    hours.value = user.daily_required_hours;
    hours.addEventListener('change', async () => {
      const before = userStateOf(user);
      const after = { ...before, daily_required_hours: Number(hours.value) || 8 };
      await UI.guard(() =>
        API.updateUser(user.id, after.name, after.daily_required_hours, after.is_active)
      );
      if (before.daily_required_hours !== after.daily_required_hours) {
        pushHistory([{ kind: 'user', op: 'update', id: user.id, before, after }]);
      }
      await reload();
    });

    const remove = UI.el('button', 'btn icon danger', '✕');
    remove.addEventListener('click', async () => {
      if (!window.confirm(I18N.t('confirm.deleteUser'))) {
        return;
      }
      await UI.guard(() => API.deleteUser(user.id));
      clearHistory();
      await reload();
    });

    line.appendChild(name);
    line.appendChild(hours);
    line.appendChild(remove);
    dom.userTable.appendChild(line);
  });

  fillOptions(
    dom.newLeaveUser,
    state.users.map((user) => String(user.id)),
    (value) => state.users.find((user) => String(user.id) === value).name
  );
  syncLeaveHours();
}

function syncLeaveHours() {
  const user = state.users.find((item) => String(item.id) === dom.newLeaveUser.value);
  const hours = user?.daily_required_hours;
  dom.newLeaveHours.value = hours > 0 ? hours : 8;
}

function renderStatuses() {
  fillOptions(
    dom.defaultStatus,
    state.statuses.map((status) => String(status.id)),
    (value) => state.statuses.find((status) => String(status.id) === value).name,
    state.settings.default_status_id
  );

  UI.clear(dom.statusTable);
  const head = row('1fr 48px 60px 28px');
  head.classList.add('head');
  head.appendChild(UI.el('span', null, I18N.t('statuses.name')));
  head.appendChild(UI.el('span', null, I18N.t('statuses.color')));
  head.appendChild(UI.el('span'));
  head.appendChild(UI.el('span'));
  dom.statusTable.appendChild(head);

  state.statuses.forEach((status) => {
    const line = row('1fr 48px 60px 28px');

    const name = document.createElement('input');
    name.className = 'input';
    name.value = status.name;
    name.addEventListener('change', async () => {
      const before = statusStateOf(status);
      const after = { ...before, name: name.value };
      await UI.guard(() => API.updateStatus(status.id, after.name, after.color));
      if (before.name !== after.name) {
        pushHistory([{ kind: 'status', op: 'update', id: status.id, before, after }]);
      }
      await reload();
    });

    const color = document.createElement('input');
    color.className = 'input';
    color.type = 'color';
    color.value = status.color;
    color.addEventListener('change', async () => {
      const before = statusStateOf(status);
      const after = { ...before, color: color.value };
      await UI.guard(() => API.updateStatus(status.id, after.name, after.color));
      if (before.color !== after.color) {
        pushHistory([{ kind: 'status', op: 'update', id: status.id, before, after }]);
      }
      await reload();
    });

    const tag = UI.el('span', 'hint', status.is_builtin ? I18N.t('statuses.builtin') : '');

    const remove = UI.el('button', 'btn icon danger', '✕');
    const isDefault = String(status.id) === state.settings.default_status_id;
    remove.disabled = isDefault;
    remove.title = isDefault ? I18N.t('statuses.cannotDeleteDefault') : '';
    remove.addEventListener('click', () => openStatusDelete(status));

    line.appendChild(name);
    line.appendChild(color);
    line.appendChild(tag);
    line.appendChild(remove);
    dom.statusTable.appendChild(line);
  });
}

function openStatusDelete(status) {
  state.pendingDelete = status.id;
  const others = state.statuses.filter((item) => item.id !== status.id);
  fillOptions(
    dom.replacementStatus,
    others.map((item) => String(item.id)),
    (value) => others.find((item) => String(item.id) === value).name
  );
  dom.statusDeleteDialog.showModal();
}

async function renderHolidays() {
  const holidays = (await UI.guard(() => API.listHolidays())) || [];
  UI.clear(dom.holidayTable);
  const head = row('110px 1fr 84px 28px');
  head.classList.add('head');
  head.appendChild(UI.el('span', null, I18N.t('calendar.date')));
  head.appendChild(UI.el('span', null, I18N.t('calendar.name')));
  head.appendChild(UI.el('span'));
  head.appendChild(UI.el('span'));
  dom.holidayTable.appendChild(head);

  if (holidays.length === 0) {
    const empty = row('1fr');
    empty.appendChild(UI.el('span', 'hint', I18N.t('common.empty')));
    dom.holidayTable.appendChild(empty);
    return;
  }

  holidays.forEach((holiday) => {
    const line = row('110px 1fr 84px 28px');
    line.appendChild(UI.el('span', 'num', holiday.date));
    line.appendChild(UI.el('span', null, holiday.name));
    line.appendChild(
      UI.el(
        'span',
        holiday.is_workday ? 'badge short' : 'badge exact',
        I18N.t(holiday.is_workday ? 'calendar.makeUp' : 'calendar.holiday')
      )
    );
    const remove = UI.el('button', 'btn icon danger', '✕');
    remove.addEventListener('click', async () => {
      await UI.guard(() => API.deleteHoliday(holiday.date));
      clearHistory();
      await renderHolidays();
    });
    line.appendChild(remove);
    dom.holidayTable.appendChild(line);
  });
}

function renderImportYears() {
  const current = new Date().getFullYear();
  const years = [current - 1, current, current + 1].map(String);
  const keep = dom.importYear.value;
  fillOptions(dom.importYear, years, (value) => value, keep || String(current));
}

const TAIWAN_CALENDAR_URL = 'https://cdn.jsdelivr.net/gh/ruyut/TaiwanCalendar/data';

function toHolidayRows(raw) {
  const rows = [];
  raw.forEach((day) => {
    const text = String(day.date ?? '');
    if (text.length !== 8) {
      return;
    }
    const date = `${text.slice(0, 4)}-${text.slice(4, 6)}-${text.slice(6, 8)}`;
    const weekend = day.week === '六' || day.week === '日';
    const description = String(day.description ?? '').trim();

    if (day.isHoliday && !weekend) {
      rows.push({ date, name: description || I18N.t('calendar.holiday'), is_workday: false });
    } else if (!day.isHoliday && weekend) {
      rows.push({ date, name: description || I18N.t('calendar.makeUp'), is_workday: true });
    }
  });
  return rows;
}

async function importTaiwanHolidays() {
  const year = Number(dom.importYear.value);
  dom.importTaiwan.disabled = true;
  const original = dom.importTaiwan.textContent;
  dom.importTaiwan.textContent = I18N.t('calendar.importing');

  try {
    const response = await fetch(`${TAIWAN_CALENDAR_URL}/${year}.json`);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    const rows = toHolidayRows(await response.json());
    const before = ((await UI.guard(() => API.listHolidays())) || [])
      .filter((item) => item.date.startsWith(`${year}-`))
      .map((item) => ({ date: item.date, name: item.name, is_workday: item.is_workday }));

    const count = await UI.guard(() => API.importHolidays(year, rows, true));
    if (count !== undefined) {
      pushHistory([{ kind: 'holidayImport', year, before, after: rows }]);
      UI.toast(I18N.t('calendar.imported', { n: count }));
      await renderHolidays();
    }
  } catch (error) {
    UI.toast(`${I18N.t('calendar.importFailed')}（${error}）`, true);
  } finally {
    dom.importTaiwan.disabled = false;
    dom.importTaiwan.textContent = original;
  }
}

async function renderLeaves() {
  const leaves = (await UI.guard(() => API.listLeaves(null))) || [];
  UI.clear(dom.leaveTable);
  const head = row('1fr 110px 56px 28px');
  head.classList.add('head');
  head.appendChild(UI.el('span', null, I18N.t('calendar.user')));
  head.appendChild(UI.el('span', null, I18N.t('calendar.date')));
  head.appendChild(UI.el('span', null, I18N.t('calendar.hours')));
  head.appendChild(UI.el('span'));
  dom.leaveTable.appendChild(head);

  if (leaves.length === 0) {
    const empty = row('1fr');
    empty.appendChild(UI.el('span', 'hint', I18N.t('common.empty')));
    dom.leaveTable.appendChild(empty);
    return;
  }

  leaves.forEach((leave) => {
    const line = row('1fr 110px 56px 28px');
    line.appendChild(UI.el('span', null, leave.user_name));
    line.appendChild(UI.el('span', 'num', leave.leave_date));
    line.appendChild(UI.el('span', 'num', UI.hours(leave.hours)));
    const remove = UI.el('button', 'btn icon danger', '✕');
    remove.addEventListener('click', async () => {
      await UI.guard(() => API.deleteLeave(leave.id));
      clearHistory();
      await renderLeaves();
    });
    line.appendChild(remove);
    dom.leaveTable.appendChild(line);
  });
}

async function renderSprint() {
  const sprint = await UI.guard(() => API.sprintSettings());
  if (!sprint) {
    return;
  }
  dom.sprintNumber.value = sprint.anchor_number ?? '';
  dom.sprintStart.value = sprint.anchor_start_date ?? '';
  dom.sprintLength.value = sprint.length_days;
}

async function renderData() {
  const path = await UI.guard(() => API.currentDbPath());
  if (path !== undefined) {
    dom.dbPath.value = path;
  }
}

function bindTabs() {
  document.querySelectorAll('.tab').forEach((tab) => {
    tab.addEventListener('click', () => {
      document.querySelectorAll('.tab').forEach((item) => item.classList.remove('active'));
      document.querySelectorAll('.panel').forEach((panel) => panel.classList.remove('active'));
      tab.classList.add('active');
      document.getElementById(`panel-${tab.dataset.panel}`).classList.add('active');
    });
  });
}

function pushHistory(changes) {
  Bridge.send('main', HISTORY_EVENT.push, { changes });
}

function clearHistory() {
  Bridge.send('main', HISTORY_EVENT.clear);
}

function settingChange(key, before, after) {
  return { kind: 'setting', key, before: before ?? '', after };
}

function userStateOf(user) {
  return {
    name: user.name,
    daily_required_hours: user.daily_required_hours,
    is_active: user.is_active
  };
}

function statusStateOf(status) {
  return { name: status.name, color: status.color };
}

async function holidayAt(date) {
  const rows = (await UI.guard(() => API.listHolidays())) || [];
  const found = rows.find((item) => item.date === date);
  return found ? { name: found.name, is_workday: found.is_workday } : null;
}

async function latestLeaveId(leave) {
  const rows = (await UI.guard(() => API.listLeaves(null))) || [];
  const matches = rows.filter(
    (item) => item.user_id === leave.user_id && item.leave_date === leave.leave_date
  );
  return matches.length > 0 ? Math.max(...matches.map((item) => item.id)) : null;
}

function bind() {
  bindTabs();

  const bindSetting = (node, key) => {
    node.addEventListener('change', async () => {
      const before = state.settings[key];
      const after = node.value;
      await UI.guard(() => API.setSetting(key, after));
      if (before !== after) {
        pushHistory([settingChange(key, before, after)]);
      }
      await reload();
    });
  };

  bindSetting(dom.language, 'language');
  bindSetting(dom.theme, 'theme');
  bindSetting(dom.density, 'density');
  bindSetting(dom.reminderRange, 'reminder_range');

  dom.addUser.addEventListener('click', async () => {
    const name = dom.newUserName.value.trim();
    if (!name) {
      return;
    }
    const hours = Number(dom.newUserHours.value) || 8;
    const id = await UI.guard(() => API.createUser(name, hours));
    if (id !== undefined) {
      pushHistory([
        {
          kind: 'user',
          op: 'create',
          id,
          before: null,
          after: { name, daily_required_hours: hours, is_active: true }
        }
      ]);
    }
    dom.newUserName.value = '';
    await reload();
  });

  bindSetting(dom.defaultStatus, 'default_status_id');

  dom.addStatus.addEventListener('click', async () => {
    const name = dom.newStatusName.value.trim();
    if (!name) {
      return;
    }
    const color = dom.newStatusColor.value;
    const id = await UI.guard(() => API.createStatus(name, color));
    if (id !== undefined) {
      pushHistory([
        { kind: 'status', op: 'create', id, before: null, after: { name, color } }
      ]);
    }
    dom.newStatusName.value = '';
    await reload();
  });

  dom.statusDeleteCancel.addEventListener('click', () => dom.statusDeleteDialog.close());
  dom.statusDeleteConfirm.addEventListener('click', async () => {
    const replacement = Number(dom.replacementStatus.value);
    await UI.guard(() => API.deleteStatus(state.pendingDelete, replacement));
    clearHistory();
    dom.statusDeleteDialog.close();
    await reload();
  });

  dom.addHoliday.addEventListener('click', async () => {
    if (!dom.newHolidayDate.value) {
      return;
    }
    const date = dom.newHolidayDate.value;
    const isWorkday = dom.newHolidayWorkday.checked;
    const fallback = I18N.t(isWorkday ? 'calendar.makeUp' : 'calendar.holiday');
    const after = { name: dom.newHolidayName.value.trim() || fallback, is_workday: isWorkday };
    const before = await holidayAt(date);
    await UI.guard(() => API.saveHoliday(date, after.name, after.is_workday));
    pushHistory([{ kind: 'holiday', date, before, after }]);
    dom.newHolidayName.value = '';
    dom.newHolidayWorkday.checked = false;
    await renderHolidays();
  });

  dom.importTaiwan.addEventListener('click', () => importTaiwanHolidays());

  dom.newLeaveUser.addEventListener('change', () => syncLeaveHours());

  dom.addLeave.addEventListener('click', async () => {
    if (!dom.newLeaveUser.value || !dom.newLeaveDate.value) {
      return;
    }
    const after = {
      user_id: Number(dom.newLeaveUser.value),
      leave_date: dom.newLeaveDate.value,
      hours: Number(dom.newLeaveHours.value) || 0,
      note: ''
    };
    const done = await UI.guard(() =>
      API.saveLeave(null, after.user_id, after.leave_date, after.hours, after.note)
    );
    if (done !== undefined) {
      const id = await latestLeaveId(after);
      if (id !== null) {
        pushHistory([{ kind: 'leave', op: 'create', id, before: null, after }]);
      }
    }
    await renderLeaves();
  });

  dom.saveSprint.addEventListener('click', async () => {
    const changes = [
      settingChange(
        'sprint_anchor_number',
        state.settings.sprint_anchor_number,
        dom.sprintNumber.value.trim()
      ),
      settingChange(
        'sprint_anchor_start_date',
        state.settings.sprint_anchor_start_date,
        dom.sprintStart.value
      ),
      settingChange(
        'sprint_length_days',
        state.settings.sprint_length_days,
        dom.sprintLength.value.trim() || '14'
      )
    ];

    for (const change of changes) {
      await UI.guard(() => API.setSetting(change.key, change.after));
    }
    if (changes.some((change) => change.before !== change.after)) {
      pushHistory(changes);
    }

    UI.toast(I18N.t('toast.saved'));
    await reload();
  });

  dom.changeDb.addEventListener('click', async () => {
    const done = await UI.guard(() => API.changeDbPath(dom.dbPath.value));
    if (done !== undefined) {
      UI.toast(I18N.t('toast.saved'));
      await reload();
    }
  });

  dom.revealDb.addEventListener('click', () => UI.guard(() => API.revealDbFolder()));

  document.addEventListener('keydown', (event) => {
    const action = historyShortcut(event);
    if (action) {
      event.preventDefault();
      Bridge.send('main', action === 'undo' ? HISTORY_EVENT.undo : HISTORY_EVENT.redo);
    }
  });

  Bridge.on(HISTORY_EVENT.changed, () => reload());
}

async function reload() {
  const bootstrap = await UI.guard(() => API.bootstrap());
  if (!bootstrap) {
    I18N.apply();
    return;
  }
  state.settings = bootstrap.settings;
  state.users = bootstrap.users;
  state.statuses = bootstrap.statuses;

  I18N.set(state.settings.language || 'zh-TW');
  UI.applyTheme(state.settings.theme || 'dark');
  UI.applyDensity(state.settings.density);
  I18N.apply();

  renderGeneral();
  renderImportYears();
  renderUsers();
  renderStatuses();
  await renderHolidays();
  await renderLeaves();
  await renderSprint();
  await renderData();
}

async function init() {
  cache();
  bind();
  await reload();
}

window.addEventListener('DOMContentLoaded', init);
