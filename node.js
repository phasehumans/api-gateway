const fs= require('fs')


setImmediate(() => {
    console.log("SetImmediate Upper");
    
})

setTimeout(() => {
    console.log("SetTimeout");
    
}, 0)

setImmediate(() => {
    console.log("SetImmediate");
    
})

fs.readFile("sample.txt", "utf-8", () => {
    setTimeout(() => {
        console.log("SetTimeout 2");
        
    }, 0)

    setImmediate(() => {
        console.log("SetImmediate 2");
        
    })
})


setImmediate(() => {
    console.log("SetImmediate Lower");
    
})

// console.log("Hello World");


/* 
Hello World
SetTimeout
SetImmediate Upper
SetImmediate
SetImmediate Lower
SetImmediate 2
SetTimeout 2
*/


/* 
w/o "hello"

SetTimeout
SetImmediate Upper
SetImmediate
SetImmediate Lower
SetImmediate 2
SetTimeout 2 

*/