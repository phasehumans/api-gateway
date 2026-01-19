// const declaration
const prn= 231107042

// variable declare: let, var
/*
prefer not to used var; scope error, use let and const

hoisting err of var

blocked scope -> let , const
fn scope -> var

*/
let fname= "chetan"
var lname= "sonawane"
location= "indave"
let hobbies

// prn= 231107040 ---> const change not allowed
// TypeError: Assignment to constant variable.
console.log(prn);

// variable can change
fname= "chaitanya"
console.log(fname);

location= "dhule"


*-9+63
// table console 
console.table([prn, fname, lname, location, hobbies])


