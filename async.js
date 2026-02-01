/* 
Async = don’t block while waiting || non blocking
Multi-threaded = do multiple things at the same time
*/

// single treaded bec. 1 DOM manipulation at a time to avoid lock

const fs = require("fs");

const data = fs.readFileSync("file.txt"); // readfile in sync
// wait till file read | blocking code
console.log(data.toString());
console.log("wait 2");

// data is undefines
// const data2= fs.readFile("file.txt", () => {
//     console.log("file read done")
//     setTimeout(() => {
//         console.log("data :" + data2);
//     }, 5000)

//     console.log("wait inside cb")
// })

const data2 = fs.readFile("filee.txt", (err, data) => {
  console.log("file read done");
  try {
    setTimeout(() => {
      console.log("data : " + data);
    }, 2000);
  } catch (error) {
    console.log(err)
  }

  console.log("wait inside cb");
});


console.log("wait 3");
// console.log(data2) >> undefine
