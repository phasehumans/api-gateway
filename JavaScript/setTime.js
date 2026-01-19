const fs= require('fs')

setTimeout(() => {
    console.log("Set TImeout")
}, 0)


setImmediate(() => {
    console.log("Set Immediate");
    
}, 0)


// console.log("Hello World");



/* 
Hello World
Set TImeout
Set Immediate
*/


/* 

flow of code chnage just because console.log()

Set Immediate
Set TImeout
*/