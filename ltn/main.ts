// import { Application } from "https://deno.land/x/abc@v1.3.3/mod.ts";
// import { gg } from "./gg.ts";

// const PORT = 13333;

// const app = new Application();

// console.log(`port ${PORT}`);

// app.get("/gg.json", (ctx) => {
//     const { "content-id": content_id_, "code-number": code_number_ } =
//         ctx.queryParams;
//     const [content_id, code_number] = [content_id_, code_number_].map(
//         (x) => parseInt(x, 10)
//     );

//     return gg(content_id, code_number);
// }).start({
//     port: PORT,
// });

import * as io from "https://deno.land/std@0.180.0/io/mod.ts";

import { gg } from "./gg.ts";

// console.log(Deno.args);

const [content_id, code_number] = Deno.args.map((x) => parseInt(x, 10));

const r = await gg(content_id, code_number);

const buf = new io.Buffer(new TextEncoder().encode(JSON.stringify(r)));

Deno.stdout.write(buf.bytes());
