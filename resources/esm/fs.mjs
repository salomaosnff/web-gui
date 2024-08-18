import { invokeAsync, invokeSync } from "lenz/ipc";

export function readFile(path) {
  return invokeAsync('fs.read', path);
}

export function readFileSync(path) {
  return invokeSync('fs.read', path);
}

export function writeFile(path, data) {
  return invokeAsync('fs.write', path, data);
}

export function writeFileSync(path, data) {
  return invokeSync('fs.write', path, data);
}