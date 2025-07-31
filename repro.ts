import * as fs from "node:fs";

const file = fs.openSync("./tests/wasm_test/temp/file.txt", "r+");
const buf = (s) => new TextEncoder().encode(s);
const b = buf("?");
console.log(b);
const stats = fs.fstatSync(file);
console.log(stats);
const size = stats.size;
fs.writeSync(file, b, 0, 1, size);
