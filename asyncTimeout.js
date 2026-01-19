function asyncTimeout(ms){
    return new Promise((resolve)=>{
        setTimeout(resolve, ms)
    })
}


async function timer(){
    console.log("inside timer async fn")

    await asyncTimeout(3000)
    console.log("1st done")

    await asyncTimeout(5000)
    console.log("2nd done")

    await asyncTimeout(7000)
    console.log("3rd done")
}


console.log("hello")
timer()
console.log("hi")


