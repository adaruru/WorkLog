const invoke = (command, args) => window.__TAURI__.core.invoke(command, args);

function report(level, message) {
  try {
    window.__TAURI__.core.invoke('log_front', { level, message: String(message) });
  } catch {
    showFatal(String(message));
  }
}

function showFatal(message) {
  let node = document.getElementById('fatal');
  if (!node) {
    node = document.createElement('pre');
    node.id = 'fatal';
    node.className = 'fatal';
    document.body.prepend(node);
  }
  node.textContent = message;
}

window.addEventListener('error', (event) => {
  const where = `${event.filename ?? '?'}:${event.lineno ?? 0}:${event.colno ?? 0}`;
  report('error', `${event.message} @ ${where}`);
  showFatal(`${event.message}\n${where}`);
});

window.addEventListener('unhandledrejection', (event) => {
  report('reject', event.reason);
  showFatal(String(event.reason));
});

const API = {
  bootstrap: () => invoke('bootstrap'),
  getSettings: () => invoke('get_settings'),
  setSetting: (key, value) => invoke('set_setting', { key, value }),

  listUsers: () => invoke('list_users'),
  createUser: (name, dailyRequiredHours) =>
    invoke('create_user', { name, dailyRequiredHours }),
  updateUser: (id, name, dailyRequiredHours, isActive) =>
    invoke('update_user', { id, name, dailyRequiredHours, isActive }),
  deleteUser: (id) => invoke('delete_user', { id }),

  listStatuses: () => invoke('list_statuses'),
  createStatus: (name, color) => invoke('create_status', { name, color }),
  updateStatus: (id, name, color) => invoke('update_status', { id, name, color }),
  deleteStatus: (id, replacementId) => invoke('delete_status', { id, replacementId }),

  searchEntries: (query) => invoke('search_entries', { query }),
  entryDetail: (id) => invoke('entry_detail', { id }),
  saveEntry: (input) => invoke('save_entry', { input }),
  setEntryStatus: (id, statusId) => invoke('set_entry_status', { id, statusId }),
  deleteEntry: (id) => invoke('delete_entry', { id }),

  linkCandidates: (entryId, keyword) => invoke('link_candidates', { entryId, keyword }),
  linkEntries: (entryId, linkedEntryId) => invoke('link_entries', { entryId, linkedEntryId }),
  unlinkEntry: (id) => invoke('unlink_entry', { id }),

  listHolidays: () => invoke('list_holidays'),
  saveHoliday: (date, name, isWorkday = false) =>
    invoke('save_holiday', { date, name, isWorkday }),
  deleteHoliday: (date) => invoke('delete_holiday', { date }),
  importHolidays: (year, items, replace) =>
    invoke('import_holidays', { year, items, replace }),

  listLeaves: (userId) => invoke('list_leaves', { userId: userId ?? null }),
  saveLeave: (id, userId, leaveDate, hours, note) =>
    invoke('save_leave', { id: id ?? null, userId, leaveDate, hours, note }),
  deleteLeave: (id) => invoke('delete_leave', { id }),

  sprintSettings: () => invoke('sprint_settings'),
  resolveRange: (scope) => invoke('resolve_range', { scope }),
  hoursReport: (userId, scope, dateFrom, dateTo) =>
    invoke('hours_report', { userId: userId ?? null, scope, dateFrom, dateTo }),

  openSettingsWindow: (theme) => invoke('open_settings_window', { theme }),
  setWindowTheme: (theme) => invoke('set_window_theme', { theme }),
  toggleDevtools: () => invoke('toggle_devtools'),
  updateTray: (tooltip, showLabel, quitLabel) =>
    invoke('update_tray', { tooltip, showLabel, quitLabel }),

  currentDbPath: () => invoke('current_db_path'),
  changeDbPath: (path) => invoke('change_db_path', { path }),
  revealDbFolder: () => invoke('reveal_db_folder')
};

const UI = {
  applyTheme(theme) {
    const value = ['dark', 'light', 'blue'].includes(theme) ? theme : 'dark';
    document.documentElement.dataset.theme = value;
    API.setWindowTheme(value).catch(() => {});
  },

  applyDensity(density) {
    const value = ['compact', 'cozy', 'comfortable'].includes(density) ? density : 'cozy';
    document.documentElement.dataset.density = value;
  },

  toast(message, isError = false) {
    let node = document.querySelector('.toast');
    if (!node) {
      node = document.createElement('div');
      node.className = 'toast';
      document.body.appendChild(node);
    }
    node.textContent = message;
    node.classList.toggle('error', isError);
    node.classList.add('show');
    clearTimeout(node._timer);
    node._timer = setTimeout(() => node.classList.remove('show'), 1800);
  },

  async guard(action) {
    try {
      return await action();
    } catch (error) {
      report('guard', error);
      UI.toast(String(error), true);
      return undefined;
    }
  },

  fatal: showFatal,

  hours(value) {
    const rounded = Math.round(value * 100) / 100;
    return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
  },

  clear(node) {
    while (node.firstChild) {
      node.removeChild(node.firstChild);
    }
  },

  el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) {
      node.className = className;
    }
    if (text !== undefined) {
      node.textContent = text;
    }
    return node;
  },

  dot(color) {
    const node = document.createElement('span');
    node.className = 'dot';
    node.style.background = color;
    return node;
  },

  today() {
    const now = new Date();
    const pad = (value) => String(value).padStart(2, '0');
    return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
  }
};

window.addEventListener('keydown', (event) => {
  if (event.key === 'F12') {
    event.preventDefault();
    API.toggleDevtools().catch(() => {});
  }
});

window.API = API;
window.UI = UI;
