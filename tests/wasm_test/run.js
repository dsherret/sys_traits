// run `deno task wasmbuild` to generate this file
import { instantiate } from "./lib/sys_traits_wasm_test.js";

const { run_tests } = await instantiate();
run_tests();
