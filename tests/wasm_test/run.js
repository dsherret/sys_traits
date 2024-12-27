// run `deno task wasmbuild` to generate this file
import { instantiate } from "./lib/sys_traits_wasm_test.generated.js";

const { run_tests } = await instantiate();
run_tests(Deno.build.os === "windows");
