import express from 'express'

const app = express ()

function calculate (n){
    const sum= 0
    for (let i= 0; i <= n; i++){
        sum= sum + i
    }

    return sum
}

app.post('/sum', (req, res) => {
    const num= req.query.n
    const ans= calculate(num)
    res.send(ans.toString())
    // always send number res in string format to avoid to get num assume has statuscode
})

app.listen(3000)