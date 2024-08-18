import { invokeSync } from "lenz/ipc";

export function emitToLabel(label, event, payload) {
  invokeSync('window.emit_label', label, event, payload);
}

export function emitToAll(event, payload) {
  invokeSync('window.emit_all', event, payload);
}

export function getWindowId() {
  return window.ID;
}

export function getAllWindows() {
  return invokeSync('window.get_all').map(id => new Window(id));
}

export class Window {
  constructor(id) {
    this.id = id;
  }

  emit(event, payload) {
    emitToLabel(this.id, event, payload);
  }

  close() {
    invokeSync('window.close', this.id);
  }

  show() {
    invokeSync('window.set_visible', this.id, true);
  }

  hide() {
    invokeSync('window.set_visible', this.id, false);
  }

  get title() {
    return invokeSync('window.get_title', this.id);
  }

  set title(title) {
    invokeSync('window.set_title', this.id, title);
  }
}

export function getWindowsByLabel(label) {
  return invokeSync('window.get_by_label', label).map(id => new Window(id));
}

export function getWindowByLabel(labels) {
  return getWindowsByLabel(labels)[0];
}

export function getMainWindow() {
  return getWindowByLabel('main');
}