let str= "chaitanya sonawane"


console.log(str.length)
// console.log(b.length())
// TypeError: b.length is not a function


console.log(str[1])

console.log(str.__proto__)  // {}
console.log(str.__proto__.__proto__)  // [Object: null prototype] {}

// eventually everything is an object in js
console.log(String.__proto__.__proto__)


// methods

console.log(String.__proto__.length)        // 0
// console.log(String.__proto__.toUpperCase())     // type error

console.log(str.length)     // blank
console.log(str.__proto__.length)        // 0
console.log(str.__proto__.__proto__.length)        // undefined


console.log(str.__proto__.toUpperCase())        // blank
// console.log(str.__proto__.__proto__.toUpperCase())        // type err : not fn
console.log(str.toUpperCase())          // CHAITANYA SONAWANE



console.log(str.split(" "))
console.log(str.replace("sona", "Sona"))