export function readFile(path) {
  return __.invokeAsync('fs.read', path);
}

export function readFileSync(path) {
  return __.invokeSync('fs.read', path);
}

export function writeFile(path, data) {
  return __.invokeAsync('fs.write', path, data);
}

export function writeFileSync(path, data) {
  return __.invokeSync('fs.write', path, data);
}