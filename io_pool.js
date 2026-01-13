const fs = require("fs");

console.log("Start");

fs.readFile("file.txt", "utf8", (err, data) => {
  console.log("File read complete");
});

setTimeout(() => {
  console.log("Timer done");
}, 0);

console.log("End");


/* 
- start
- end
- file read done
- timer done


*/