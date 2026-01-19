// primitive datatype: num, str, boolean
let x= 10
let y= 20

console.log(x)
console.log(y)

let a= 33
let b= a

console.log(a)
console.log(b)

a= 40
console.log(a) // 40
console.log(b) // 33

b= 44
console.log(a)  // 40
console.log(b)  // 44


// non-primitive datatype: array & object
let rollno = [1,2,3]
let rollno2= rollno
console.log(rollno)
console.log(rollno2)

rollno[0] = 69
console.log(rollno)         //both get change [0] = 69
console.log(rollno2)