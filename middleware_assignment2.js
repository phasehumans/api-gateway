const express= require('express')

const app= express()

const requestCount= 0

function reqCount(req, res, next){
    requestCount++
    next()
}

app.get('/hi', reqCount, (req, res) =>{
    res.json({
        msg: "hi"
    })
})

app.get('/hello', reqCount, (req, res) =>{
    res.json({
        msg: "hello"
    })
})

app.get('/getcount', (req, res) => {
    console.log(requestCount)
    res.json({
        req: requestCount
    })
})


app.listen(3000)