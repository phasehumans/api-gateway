// /add/:a/:b
// req.params.a



const express= require('express')

const app= express()

// query 127.0.0.1:3000/sum?a=2&b=3
app.get('/sum', (req, res) => {
    const num1= parseInt(req.query.a)
    const num2= parseInt(req.query.b)

    res.json({
        ans: num1 + num2
    })
})

// params 127.0.0.1:3000/sum2/10/10
app.get('/sum2/:a/:b', (req, res) =>{
    const n1= parseInt(req.params.a)
    const n2= parseInt(req.params.b)

    res.json({
        ans: n1 + n2
    })
})

app.listen(3000)