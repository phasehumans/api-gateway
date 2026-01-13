const fs= require('fs')
//  w/o . checks built in module

const x= require('./math')
// w/. checks exports{}

fs.writeFile('./sample.txt', "Hello World", () => {})

console.log({__filename, __dirname})

console.log(x.add(3, 7));