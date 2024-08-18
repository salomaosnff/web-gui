import { invokeAsync, invokeSync } from 'lenz/ipc';

export function show(options) {
  invokeAsync('dialog.show', {
    title: options.title ?? 'Mensagem',
    message: options.message ?? '',
    level: options.level ?? 'info',
  });
}

export function info(options) {
  show({
    ...options,
    level: 'info',
  });
}

export function warn(options) {
  show({
    ...options,
    level: 'warn',
  });
}

export function error(options) {
  show({
    ...options,
    level: 'error',
  });
}

export function confirm(options) {
  return invokeSync('dialog.confirm', {
    title: options.title ?? 'Confirmação',
    message: options.message ?? '',
    level: options.level ?? 'info',
  });
}

export function openFile(options) {
  return invokeSync('dialog.files.open', {
    title: options.title ?? 'Abrir Arquivo',
    filters: options.filters ?? {},
    multiple: false,
  });
}

export function openFiles(options) {
  return invokeSync('dialog.files.open', {
    title: options.title ?? 'Abrir Arquivos',
    filters: options.filters ?? {},
    multiple: true,
  });
}

export function saveFile(options) {
  return invokeSync('dialog.files.save', {
    title: options.title ?? 'Salvar Arquivo',
    filters: options.filters ?? {},
  });
}

export function selectFolder(options) {
  return invokeSync('dialog.folder.select', {
    title: options.title ?? 'Selecionar Pasta',
    multiple: false,
  });
}

export function selectFolders(options) {
  return invokeSync('dialog.folder.select', {
    title: options.title ?? 'Selecionar Pastas',
    multiple: true,
  });
}