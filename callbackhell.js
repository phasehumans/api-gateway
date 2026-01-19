// setTimeout callback hell:

// one async operation after another
setTimeout(function (){
    console.log("printed after 3 sec")
    setTimeout(()=>{
        console.log("printed after 5 sec")
        setTimeout(() =>{
            console.log("printed after 7 sec")
        }, 7000)
    }, 5000)
}, 3000)

console.log("outside the callbck hell")