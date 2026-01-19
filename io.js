import fs, { readFile } from "fs"

const data= fs.readFileSync("sample.txt", "utf-8")
// const data= fs.readFile("sample.txt", "utf-8")       --> reads by asyn


console.log(data)

const anotherData= fs.readFileSync("sampleTwo.txt", "utf-8")            // synchronously
console.log(anotherData)

