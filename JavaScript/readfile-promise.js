const fs = require("fs")

/* 
readfile will have obj of promise, readFile().then
readTheFile will have actual logic and resolve of promise

*/

function readTheFile (resolve){
    console.log("i am inside readTheFile")
    console.log(resolve)
    fs.readFile("sample.txt", "utf-8", function(err, data){
        resolve(data)
    })
    console.log(resolve)
}

function readFile (filename){
    // read the file and return its value
    console.log("i am inside readFile")
    return new Promise(readTheFile)
}


// using the promise    

// const a= readFile()

// a.then((content)=>{
//     console.log("i am inside then wala fn")
//     console.log(content)
// })


readFile().then((data) =>{
    console.log(data)
})