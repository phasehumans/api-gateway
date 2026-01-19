const { program } = require("commander");

const fs= require('fs')

program.option("--first").option("-s, --separator <char>").argument("<string>");

program.parse();

// const options = program.opts();
// const limit = options.first ? 1 : undefined;
// console.log(program.args[0].split(options.separator, limit));


/* 
node -h : h is helper fn


- cli can be done w/ process.argv[index]
*/



function readFileProm (filepath){
    return new Promise((resolve) =>{
        fs.readFile(filepath, 'utf-8', (err, data) =>{
            if(err){
                console.log("error while reading file")
                return resolve()
                // if err,  still resolve
            }
            let count= 0
            const content= data 
            for(char in content){
                count++
            }

            console.log(count)

            resolve()
        })
    })
}

readFileProm(program.args[0])
// program.args[0]
