const crypto = require("crypto");

process.env.UV_THREADPOOL_SIZE = 2; // only 2 worker threads

const start = Date.now();

for (let i = 1; i <= 4; i++) {
  crypto.pbkdf2("password", "salt", 100000, 64, "sha512", () => {
    console.log(`[${Date.now() - start}ms] -> Task ${i} done`);
  });
}
