const express= require('express')

const app = express()

app.get('add', (req, res) => {
    // query parameters
    
    // const num1= req.query.a
    // const num2= req.query.b

    const num1= parseInt(req.query.a)
    const num2= parseInt(req.query.b)

    // const ans = num1 + num2
    // res.send(ans)

    res.json({
        answer : num1 + num2
    })
})

app.get('sub', (req, res) => {
    const num1= req.query.a
    const num2= req.query.b

    res.json({
      answer: num1 - num2,
    });
})

app.get('multi', (req, res) => {
    const num1= req.query.a
    const num2= req.query.b

    res.json({
      answer: num1 * num2,
    });
})

app.get('div', (req, res) => {
    const num1= req.query.a
    const num2= req.query.b

    res.json({
      answer: num1 / num2,
    });
})


app.listen(3000)