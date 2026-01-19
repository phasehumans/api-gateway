function abc() {
    console.log("this is abc function")
}


function higherOreder (cb){
    console.log("inside higher order fn")
    // abc()
    cb()
}

// higherOreder()          // TypeError: cb is not a function
higherOreder(abc)