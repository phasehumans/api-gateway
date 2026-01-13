let arr = [1, 2, 3, 4]
let arr2 = arr
// memory address ref is passed in arr2

console.log(arr)
console.log(arr2)

arr2[2] = 99

console.log(arr)
console.log(arr2)