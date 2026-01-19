/* 
- js in node: single threaded + thread pool & libuv
- js runtime env (setup that runs code)
- built w/ v8 engine & libuv (non blocking I/O)
- c++ bindings for OS level fn/ sys call
- alternative bun, deno
- use for SSR (web page generated on server and sent), opp to client side rendering
- v8 engine: JS code → Parser → AST → Ignition (bytecode) → TurboFan (optimized machine code) → CPU executes
- lifecycle: Run → Wrap → Parse/Compile → Execute → Async to libuv → Event Loop + Microtasks → Exit
- single thread: one thing at a time
- thread pool (4): run blocking tasks (heavy blocking tasks, file, crypto, compression) so main thread stays non blocking, background workers in nodejs


*/