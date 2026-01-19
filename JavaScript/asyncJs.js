import fs from "fs"

fs.readFile("sample.txt", "utf-8", (err, data) =>{
    console.log(data)
} )

// const data= fs.readFileSync("sampleTwo.txt", "utf-8")
// console.log(data)

console.log("chaitanya")