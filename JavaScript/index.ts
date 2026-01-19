
function greet(name : string){
    console.log(`hello, good morning ${name}`);
}

greet("Chaitanya")


function sumF(num1: number, num2: number){
    return num1 + num2
}

console.log(sumF(2, 3))


function isLegal(age: number): boolean{
    if (age > 18){
        return true
    }else{
        return false
    }
}

console.log(isLegal(19));


function delayFn(fn: () => void){
    setTimeout(fn, 1000)
}

delayFn(function(){
    console.log("hello there")
})