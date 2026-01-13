const express= require('express')

const app= express()


function logDetails(req, res, next){
    console.log("method: " + req.method)
    // console.log("URL: "+ req.url) --> logs route
    console.log(req.hostname)
    console.log(new Date())
    next()
}

// app.use(logDetails) --> to add this middlware to routes below it

app.get('/sayhi', logDetails, (req, res) =>{
    res.json({
        msg: "hello user"
    })
})

app.listen(3000)