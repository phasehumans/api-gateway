// promise takes fn as input whose 1st args is fn, which is called after resolving the fn 

function random (resolve){
    resolve()
    // call of then(cb) depnd upon call of resolve fn
}

let a = new Promise(random)

// promise class takes random , and random first args is fn resolve

// console.log(a)


// using eventual value returned by the promise
function cb (){
    console.log("promise succeded")
}

a.then(cb)

// console.log(a)


/* 
Attaches a callback that is invoked when the Promise is settled (fulfilled or rejected). The
     * resolved value cannot be modified from the callback.
*/