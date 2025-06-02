// run `deno task wasmbuild` to generate this file
import { instantiate } from "./lib/sys_traits_wasm_test.generated.js";

const original = Deno.symlinkSync;
Deno.symlinkSync = function(a, b, c) {
  console.log(a);
  console.log(b);
  console.log(c);
  original(a, b, c);
};

const { run_tests } = await instantiate();
run_tests();
