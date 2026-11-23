const HISTORY_LIMIT = 20;

const HISTORY_EVENT = {
  push: 'worklog://history-push',
  undo: 'worklog://history-undo',
  redo: 'worklog://history-redo',
  clear: 'worklog://history-clear',
  changed: 'worklog://history-changed'
};

function isFilterOnly(item) {
  return item.changes.length === 1 && item.changes[0].kind === 'filter';
}

const History = {
  undoStack: [],
  redoStack: [],

  push(item) {
    const last = this.undoStack[this.undoStack.length - 1];
    if (last && isFilterOnly(last) && isFilterOnly(item)) {
      last.changes[0].after = item.changes[0].after;
    } else {
      this.undoStack.push(item);
      if (this.undoStack.length > HISTORY_LIMIT) {
        this.undoStack.shift();
      }
    }
    this.redoStack.length = 0;
  },

  canUndo() {
    return this.undoStack.length > 0;
  },

  canRedo() {
    return this.redoStack.length > 0;
  },

  takeUndo() {
    const item = this.undoStack.pop();
    if (item) {
      this.redoStack.push(item);
    }
    return item;
  },

  takeRedo() {
    const item = this.redoStack.pop();
    if (item) {
      this.undoStack.push(item);
    }
    return item;
  },

  clear() {
    this.undoStack.length = 0;
    this.redoStack.length = 0;
  }
};

const Bridge = {
  api() {
    const tauri = window.__TAURI__;
    return tauri && tauri.event ? tauri.event : null;
  },

  async send(target, name, payload) {
    const api = this.api();
    if (!api) {
      return;
    }
    try {
      await api.emitTo(target, name, payload ?? {});
    } catch (error) {
      report('bridge', error);
    }
  },

  async on(name, handler) {
    const api = this.api();
    if (!api) {
      return;
    }
    try {
      await api.listen(name, (event) => handler(event.payload));
    } catch (error) {
      report('bridge', error);
    }
  }
};

function historyShortcut(event) {
  if (!event.ctrlKey || event.altKey) {
    return null;
  }
  const key = event.key.toLowerCase();
  if (key !== 'z' && key !== 'y') {
    return null;
  }
  const node = event.target;
  const tag = node && node.tagName ? node.tagName.toLowerCase() : '';
  if (tag === 'input' || tag === 'textarea') {
    return null;
  }
  if (key === 'y' || event.shiftKey) {
    return 'redo';
  }
  return 'undo';
}

window.History = History;
window.HISTORY_EVENT = HISTORY_EVENT;
window.Bridge = Bridge;
window.historyShortcut = historyShortcut;
