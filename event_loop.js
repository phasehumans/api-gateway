/* 
- event loop decides what to run next
- runs on single thread
- executes top level code (syn code)
- event callbacks register
- event loop starts
    - expired timer callbacks ---> setTimeout, setInterval
    - IO pooling -- input/ output cb
    - setImmediate cb
    - close cb (clean up fn)
    - checks for pending cb, if yes then repeat event loop
    - else exit event loop


*/

setTimeout(() => {
    console.log("SetTimeout");
    
}, 0)

setImmediate(() => {
    console.log("SetImmediate");
    
})


// console.log("Hello World");


/* 
Hello World
SetTimeout
SetImmediate

log executes, setTimeout fn registers and timer starts

- setImediate always verfiy for if any timeout has completed, if completed then setImmediate waits

----

w/o console.log("hello world")
SetImmediate
SetTimeout

setTimeout fn register, but no timer expired, so setImmediate executes first
*/
